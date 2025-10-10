use std::sync::{Arc, RwLock};

use ::tracing::info;
use clap::Parser;
use config::{Config, ConfigError};
use http::ApiServer;
use langdb_core::metadata::error::DatabaseError;
use langdb_core::metadata::services::model::ModelServiceImpl;
use langdb_core::{error::GatewayError, usage::InMemoryStorage};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod callback_handler;
mod cli;
mod config;
mod cost;
mod guardrails;
mod handlers;
mod http;
mod limit;
mod middleware;
mod run;
mod seed;
mod session;
mod tracing;
mod tui;
mod usage;
use langdb_core::events::broadcast_channel_manager::BroadcastChannelManager;
use static_serve::embed_asset;
use static_serve::embed_assets;
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
    DatabaseError(#[from] DatabaseError),
    #[error(transparent)]
    ModelsLoadError(#[from] run::models::ModelsLoadError),
    #[error(transparent)]
    ProvidersLoadError(#[from] run::providers::ProvidersLoadError),
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

embed_assets!("dist", compress = true);

#[actix_web::main]
async fn main() -> Result<(), CliError> {
    dotenv::dotenv().ok();
    println!("{LOGO}");
    std::env::set_var("RUST_BACKTRACE", "1");

    let cli = cli::Cli::parse();

    let db_pool = init_db()?;

    langdb_core::metadata::utils::init_db(&db_pool);

    let project_trace_senders = Arc::new(BroadcastChannelManager::new(Default::default()));

    let project_trace_senders_cleanup = Arc::clone(&project_trace_senders);
    langdb_core::events::broadcast_channel_manager::start_cleanup_task(
        (*project_trace_senders_cleanup).clone(),
    );

    tracing::init_tracing(project_trace_senders.inner().clone());
    // Seed the database with a default project if none exist
    seed::seed_database(&db_pool)?;

    match cli
        .command
        .unwrap_or(cli::Commands::Serve(cli::ServeArgs::default()))
    {
        cli::Commands::Login => session::login().await,
        cli::Commands::Sync => {
            tracing::init_tracing(project_trace_senders.inner().clone());
            info!("Syncing models from API to database...");
            let models = run::models::fetch_and_store_models(db_pool.clone()).await?;
            info!("Successfully synced {} models to database", models.len());
            Ok(())
        }
        cli::Commands::SyncProviders => {
            tracing::init_tracing(project_trace_senders.inner().clone());
            info!("Syncing providers from API to database...");
            run::providers::sync_providers(db_pool.clone()).await?;
            info!("Successfully synced providers to database");
            Ok(())
        }
        cli::Commands::List => {
            tracing::init_tracing(project_trace_senders.inner().clone());
            // Query models from database
            use langdb_core::metadata::services::model::ModelService;
            let model_service = ModelServiceImpl::new(db_pool.clone());
            let db_models = model_service.list(None)?;

            info!("Found {} models in database\n", db_models.len());

            // Convert DbModel to ModelMetadata and display as table
            let models: Vec<langdb_core::models::ModelMetadata> =
                db_models.into_iter().map(|m| m.into()).collect();

            run::table::pretty_print_models(models);
            Ok(())
        }
        cli::Commands::Serve(serve_args) => {
            // Check if models table is empty and sync if needed
            seed::seed_models(&db_pool).await?;

            // Check if providers table is empty and sync if needed
            seed::seed_providers(&db_pool).await?;

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
                let api_server = ApiServer::new(config, db_pool.clone());
                let model_service = Arc::new(Box::new(ModelServiceImpl::new(db_pool.clone()))
                    as Box<dyn langdb_core::metadata::services::model::ModelService + Send + Sync>);
                let server_handle = tokio::spawn(async move {
                    match api_server
                        .start(
                            Some(storage_clone),
                            model_service,
                            project_trace_senders.clone(),
                        )
                        .await
                    {
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
                let config = Config::load(&cli.config)?;
                let config = config.apply_cli_overrides(&cli::Commands::Serve(serve_args));
                let api_server = ApiServer::new(config, db_pool.clone());
                let model_service = Arc::new(Box::new(ModelServiceImpl::new(db_pool.clone()))
                    as Box<dyn langdb_core::metadata::services::model::ModelService + Send + Sync>);
                let server_handle = tokio::spawn(async move {
                    let storage = Arc::new(Mutex::new(InMemoryStorage::new()));
                    match api_server
                        .start(Some(storage), model_service, project_trace_senders.clone())
                        .await
                    {
                        Ok(server) => server.await,
                        Err(e) => Err(e),
                    }
                });

                let frontend_handle = tokio::spawn(async move {
                    let index = embed_asset!("dist/index.html");
                    let router = static_router()
                        .fallback(index);
                      
                    let listener = tokio::net::TcpListener::bind("0.0.0.0:8084").await.unwrap();
                    axum::serve(listener, router.into_make_service())
                        .await
                        .unwrap();
                });

                tokio::select! {
                    r = server_handle => {
                        if let Err(e) = r {
                            eprintln!("Counter loop error: {e}");
                        }
                    }
                    r = frontend_handle => {
                        if let Err(e) = r {
                            eprintln!("Server error: {e}");
                        }
                    }
                }
            }
            Ok(())
        }
    }
}

fn init_db() -> Result<langdb_core::metadata::pool::DbPool, CliError> {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let ellora_dir = format!("{home_dir}/.ellora");
    std::fs::create_dir_all(&ellora_dir).unwrap_or_default();
    let ellora_db_file = format!("{ellora_dir}/ellora.sqlite");
    let db_pool = langdb_core::metadata::pool::establish_connection(ellora_db_file, 10);

    langdb_core::metadata::utils::init_db(&db_pool);

    Ok(db_pool)
}
