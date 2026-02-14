//! MCP Server implementation for ElevenLabs CLI.
//!
//! Minimal working MCP server with stdio transport.

use anyhow::Result;
use rmcp::{
    ServerHandler, ServiceExt,
    model::{ServerCapabilities, ServerInfo, CallToolResult, Content},
    transport::stdio,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

/// ElevenLabs MCP Server
#[derive(Clone)]
pub struct ElevenLabsMcpServer {
    api_key: Arc<RwLock<Option<String>>>,
}

impl ElevenLabsMcpServer {
    pub fn new() -> Self {
        Self {
            api_key: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Arc::new(RwLock::new(Some(api_key)));
        self
    }
}

impl ServerHandler for ElevenLabsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("ElevenLabs CLI - 50+ MCP tools. Use CLI for full functionality.".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}

pub async fn run_server() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let config = Config::load().unwrap_or_default();
    let api_key = config.api_key.unwrap_or_else(|| "".to_string());

    let server = ElevenLabsMcpServer::new().with_api_key(api_key);

    tracing::info!("Starting ElevenLabs MCP server...");
    
    let service = server.serve(stdio()).await?;
    tracing::info!("MCP server started. Waiting for connections...");
    
    service.waiting().await?;
    
    Ok(())
}
