//! MCP (Model Context Protocol) server implementation for ElevenLabs CLI.
//!
//! This module exposes all ElevenLabs API functionality as MCP tools that can be
//! used by AI assistants like Claude, GPT-4, etc.

#[cfg(feature = "mcp")]
pub mod server;

#[cfg(feature = "mcp")]
pub mod tools;

/// Output formatting options for MCP responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text
    Text,
    /// JSON structured output
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Text
    }
}

/// Configuration for MCP server behavior
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// Whether to include verbose information in responses
    pub verbose: bool,
    /// Default output format
    pub output_format: OutputFormat,
    /// Maximum text length for TTS (to prevent abuse)
    pub max_tts_text_length: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            output_format: OutputFormat::Text,
            max_tts_text_length: 5000,
        }
    }
}

/// Run the MCP server (only available with mcp feature)
#[cfg(feature = "mcp")]
pub async fn run_server(enable_tools: Option<&str>, disable_tools: Option<&str>) -> anyhow::Result<()> {
    server::run_server(enable_tools, disable_tools).await
}

#[cfg(not(feature = "mcp"))]
pub async fn run_server(_enable_tools: Option<&str>, _disable_tools: Option<&str>) -> anyhow::Result<()> {
    anyhow::bail!("MCP support not compiled in. Rebuild with --features mcp")
}
