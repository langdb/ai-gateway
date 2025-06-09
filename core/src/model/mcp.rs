use std::collections::HashMap;

use regex::Regex;
use rmcp::model::{
    CallToolRequest, CallToolRequestMethod, ClientRequest, Extensions, GetMeta, ServerResult,
};
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::ServiceError;
use rmcp::{model::CallToolRequestParam, transport::SseClientTransport, RoleClient};
use tracing::debug;

use crate::types::gateway::{McpDefinition, McpTool, McpTransportType, ServerTools, ToolsFilter};
use rmcp::service::DynService;
use rmcp::service::RunningService;
use rmcp::service::ServiceExt;

#[derive(Debug, thiserror::Error)]
pub enum McpServerError {
    #[error("Invalid server name: {0}")]
    InvalidServerName(String),

    #[error("Server initialization error: {0}")]
    ServerInitializeError(#[from] Box<rmcp::service::ServerInitializeError<std::io::Error>>),

    #[error("SSE transport error: {0}")]
    SseTransportError(#[from] rmcp::transport::sse_client::SseTransportError<reqwest::Error>),

    #[error("Client initialization error: {0}")]
    ClientInitializeError(#[from] Box<rmcp::service::ClientInitializeError<std::io::Error>>),

    #[error("Service error: {0}")]
    ServiceError(#[from] rmcp::ServiceError),

    #[error("Client start error: {0}")]
    ClientStartError(String),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    BoxedError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    StdIOError(#[from] std::io::Error),

    #[error(transparent)]
    ParseError(#[from] serde_json::Error),

    #[error("No text content in tool {0} result")]
    NoTextInToolResult(String),

    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

impl From<rmcp::service::ClientInitializeError<std::io::Error>> for McpServerError {
    fn from(value: rmcp::service::ClientInitializeError<std::io::Error>) -> Self {
        McpServerError::ClientInitializeError(Box::new(value))
    }
}

impl From<rmcp::service::ServerInitializeError<std::io::Error>> for McpServerError {
    fn from(value: rmcp::service::ServerInitializeError<std::io::Error>) -> Self {
        McpServerError::ServerInitializeError(Box::new(value))
    }
}

fn _validate_server_name(name: &str) -> Result<(), McpServerError> {
    match name {
        "websearch" | "Web Search" => Ok(()),
        _ => Err(McpServerError::InvalidServerName(name.to_string())),
    }
}

async fn _cache_durationasync_server(name: &str) -> Result<(), McpServerError> {
    match name {
        "websearch" | "Web Search" => {
            // let async_rw = OneshotTransport::new(tokio::io::stdin(), tokio::io::stdout());
            // serve_server(tavily::TavilySearch {}, async_rw).await?;

            Ok(())
        }
        _ => Err(McpServerError::InvalidServerName(name.to_string())),
    }
}

pub fn stdio() -> (tokio::io::Stdin, tokio::io::Stdout) {
    (tokio::io::stdin(), tokio::io::stdout())
}

pub async fn get_transport(
    definition: &McpDefinition,
) -> Result<RunningService<RoleClient, Box<dyn DynService<RoleClient>>>, McpServerError> {
    match &definition.r#type {
        McpTransportType::Sse { server_url, .. } => {
            let transport = SseClientTransport::start(server_url.clone()).await?;

            Ok(()
                .into_dyn()
                .serve(transport)
                .await
                .map_err(|e| McpServerError::ClientStartError(e.to_string()))?)
        }
        McpTransportType::Http { server_url, .. } => {
            let transport = StreamableHttpClientTransport::from_uri(server_url.clone());

            Ok(()
                .into_dyn()
                .serve(transport)
                .await
                .map_err(|e| McpServerError::ClientStartError(e.to_string()))?)
        }
        McpTransportType::InMemory { name, .. } => {
            Err(McpServerError::InvalidServerName(name.clone()))
            // validate_server_name(&name)?;
            // let name = name.clone();
            // tokio::spawn(async move {
            //     async_server(&name).await.unwrap()
            // });

            // Ok(()
            //     .into_dyn()
            //     .serve(InMemoryTransport::new())
            //     .await
            //     .map_err(|e| McpServerError::ClientStartError(e.to_string()))?)
        }
        _ => Err(McpServerError::InvalidServerName(
            "Invalid or unsupported server type".to_string(),
        )),
    }
}

pub async fn get_tools(definitions: &[McpDefinition]) -> Result<Vec<ServerTools>, McpServerError> {
    let mut all_tools = Vec::new();

    for tool_def in definitions {
        let mcp_server_name = tool_def.server_name();
        let client = get_transport(tool_def).await?;
        let tools = client.list_tools(Default::default()).await?;
        client.cancel().await?;

        let mut tools = tools.tools;
        let total_tools = tools.len();

        // Filter tools based on actions_filter if specified
        match &tool_def.filter {
            ToolsFilter::All => {
                tracing::debug!("Loading all {} tools from {}", total_tools, mcp_server_name);
            }
            ToolsFilter::Selected(selected) => {
                let before_count = tools.len();
                tools.retain_mut(|tool| {
                    let found = selected.iter().find(|t| {
                        if tool.name == t.name {
                            true
                        } else if let Ok(name_regex) = Regex::new(&t.name) {
                            debug!("Matching {} against pattern {}", tool.name, t.name);
                            name_regex.is_match(&tool.name)
                        } else {
                            false
                        }
                    });
                    if let Some(Some(d)) = found.as_ref().map(|t| t.description.as_ref()) {
                        tool.description = Some(d.clone().into());
                    }
                    found.is_some()
                });
                tracing::debug!(
                    "Filtered tools for {}: {}/{} tools selected",
                    mcp_server_name,
                    tools.len(),
                    before_count
                );
            }
        }

        let mcp_tools = tools
            .into_iter()
            .map(|t| McpTool(t, tool_def.clone()))
            .collect();

        all_tools.push(ServerTools {
            tools: mcp_tools,
            definition: tool_def.clone(),
        });
    }

    tracing::debug!("Loaded {} tool definitions in total", all_tools.len());
    Ok(all_tools)
}

pub async fn get_raw_tools(
    definitions: &McpDefinition,
) -> Result<Vec<rmcp::model::Tool>, McpServerError> {
    let client = get_transport(definitions).await?;
    let tools = client.list_tools(Default::default()).await?;
    client.cancel().await?;

    Ok(tools.tools)
}

pub async fn execute_mcp_tool(
    def: &McpDefinition,
    tool: &rmcp::model::Tool,
    inputs: HashMap<String, serde_json::Value>,
    meta: Option<serde_json::Value>,
) -> Result<String, McpServerError> {
    let name = tool.name.clone();

    let client = get_transport(def).await?;

    let mut args = serde_json::Map::new();

    for (key, value) in inputs {
        args.insert(key, value);
    }

    let params = CallToolRequestParam {
        name: tool.name.clone(),
        arguments: Some(args),
    };
    let mut t = ClientRequest::CallToolRequest(CallToolRequest {
        method: CallToolRequestMethod,
        params,
        extensions: Extensions::default(),
    });
    if let Some(meta) = meta {
        if let Some(map) = meta.as_object() {
            for (key, value) in map {
                t.get_meta_mut().insert(key.clone(), value.clone());
            }
        }
    }

    let response = client.send_request(t).await?;
    client.cancel().await?;

    let response = match response {
        ServerResult::CallToolResult(result) => Ok(result),
        _ => Err(ServiceError::UnexpectedResponse),
    }?;

    // Extract text from the response
    if !response.content.is_empty() {
        // Try to extract text from the first content item
        if let Some(content) = response.content.first() {
            // Access text content from the raw field
            if let Some(text) = content.raw.as_text().map(|t| t.text.clone()) {
                tracing::debug!("Tool {name}: execution completed successfully", name = name);
                return Ok(text);
            }
        }
    }

    tracing::error!("Tool {name}: No text content in tool response", name = name);
    Err(McpServerError::NoTextInToolResult(name.to_string()))
}
