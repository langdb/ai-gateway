use std::sync::{Arc, RwLock};

use clap::Parser;
use config::{Config, ConfigError};
use http::ApiServer;
use langdb_core::{error::GatewayError, usage::InMemoryStorage};
use uuid::Uuid;
use run::models::{load_models, ModelsLoadError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use langdb_metadata::pool::DbPool;
use langdb_metadata::services::project::{ProjectService, ProjectServiceImpl};
use langdb_metadata::error::DatabaseError;
use langdb_metadata::models::project::NewProjectDTO;
use ::tracing::info;

mod callback_handler;
mod cli;
mod config;
mod cost;
mod guardrails;
mod handlers;
mod http;
mod limit;
mod middleware;
mod otel;
mod run;
mod session;
mod test_utils;
mod tracing;
mod tui;
mod usage;
use tokio::sync::Mutex;
use tui::{Counters, Tui};

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    GatewayError(#[from] Box<GatewayError>),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    ServerError(#[from] http::ServerError),
    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    #[error(transparent)]
    ModelsError(#[from] ModelsLoadError),
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    api_key: String,
}

pub const LOGO: &str = r#"

  ██       █████  ███    ██  ██████  ██████  ██████  
  ██      ██   ██ ████   ██ ██       ██   ██ ██   ██ 
  ██      ███████ ██ ██  ██ ██   ███ ██   ██ ██████  
  ██      ██   ██ ██  ██ ██ ██    ██ ██   ██ ██   ██ 
  ███████ ██   ██ ██   ████  ██████  ██████  ██████
"#;

/// Seeds the database with a default project if no projects exist
fn seed_database(db_pool: &DbPool) -> Result<(), CliError> {
    let project_service = ProjectServiceImpl::new(Arc::new(db_pool.clone()));
    
    // Use a dummy owner_id for seeding (you might want to change this)
    let dummy_owner_id = Uuid::nil();
    
    // Check if any projects exist
    let project_count = project_service.count(dummy_owner_id)?;
    
    if project_count == 0 {
        info!("No projects found in database. Creating default project...");
        
        let default_project = NewProjectDTO {
            name: "Default Project".to_string(),
            description: Some("Default project created during database seeding".to_string()),
            settings: None,
            private_model_prices: None,
            usage_limit: None,
        };
        
        let created_project = project_service.create(default_project, dummy_owner_id)?;
        info!("Created default project: {} (ID: {})", created_project.name, created_project.id);
    } else {
        info!("Found {} existing projects in database", project_count);
    }
    
    Ok(())
}

#[actix_web::main]
async fn main() -> Result<(), CliError> {
    dotenv::dotenv().ok();
    println!("{LOGO}");
    std::env::set_var("RUST_BACKTRACE", "1");

    let cli = cli::Cli::parse();

    let db_pool = langdb_metadata::pool::establish_connection("langdb.sqlite".to_string(), 10);

    langdb_metadata::utils::init_db(&db_pool);
    
    // Seed the database with a default project if none exist
    seed_database(&db_pool)?;

    match cli
        .command
        .unwrap_or(cli::Commands::Serve(cli::ServeArgs::default()))
    {
        cli::Commands::Login => session::login().await,
        cli::Commands::Update { force } => {
            tracing::init_tracing();
            println!("Updating models{}...", if force { " (forced)" } else { "" });
            let models = load_models(true).await?;
            println!("{} Models updated successfully!", models.len());
            Ok(())
        }
        cli::Commands::List => {
            tracing::init_tracing();
            println!("Available models:");
            let models = load_models(false).await?;
            run::table::pretty_print_models(models);
            Ok(())
        }
        cli::Commands::Serve(serve_args) => {
            if serve_args.interactive {
                let storage = Arc::new(Mutex::new(InMemoryStorage::new()));
                let storage_clone = storage.clone();
                let counters = Arc::new(RwLock::new(Counters::default()));
                let counters_clone = counters.clone();

                let (log_sender, log_receiver) = tokio::sync::mpsc::channel(100);
                tracing::init_tui_tracing(log_sender);

                let counter_handle =
                    tokio::spawn(async move { Tui::spawn_counter_loop(storage, counters).await });

                let config = Config::load(&cli.config)?;
                let config = config.apply_cli_overrides(&cli::Commands::Serve(serve_args));
                let api_server = ApiServer::new(config, Arc::new(db_pool.clone()));
                let models = load_models(false).await?;
                let server_handle = tokio::spawn(async move {
                    match api_server.start(models, Some(storage_clone)).await {
                        Ok(server) => server.await,
                        Err(e) => Err(e),
                    }
                });

                let tui_handle = tokio::spawn(async move {
                    let tui = Tui::new(log_receiver);
                    if let Ok(mut tui) = tui {
                        tui.run(counters_clone).await?;
                    }
                    Ok::<(), CliError>(())
                });

                // Create abort handles
                let counter_abort = counter_handle.abort_handle();
                let server_abort = server_handle.abort_handle();

                tokio::select! {
                    r = counter_handle => {
                        if let Err(e) = r {
                            eprintln!("Counter loop error: {e}");
                        }
                    }
                    r = server_handle => {
                        if let Err(e) = r {
                            eprintln!("Server error: {e}");
                        }
                    }
                    r = tui_handle => {
                        if let Err(e) = r {
                            eprintln!("TUI error: {e}");
                        }
                        // If TUI exits, abort other tasks
                        counter_abort.abort();
                        server_abort.abort();
                    }
                }
            } else {
                tracing::init_tracing();

                let config = Config::load(&cli.config)?;
                let config = config.apply_cli_overrides(&cli::Commands::Serve(serve_args));
                let api_server = ApiServer::new(config, Arc::new(db_pool.clone()));
                let models = load_models(false).await?;
                let server_handle = tokio::spawn(async move {
                    let storage = Arc::new(Mutex::new(InMemoryStorage::new()));
                    match api_server.start(models, Some(storage)).await {
                        Ok(server) => server.await,
                        Err(e) => Err(e),
                    }
                });

                match server_handle.await {
                    Ok(result) => {
                        if let Err(e) = result {
                            eprintln!("{e}");
                        }
                    }
                    Err(e) => eprintln!("{e}"),
                }
            }
            Ok(())
        }
    }
}