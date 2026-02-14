//! Tools CLI arguments

use clap::{Args, Subcommand};

/// Tools API arguments
#[derive(Args)]
pub struct ToolsArgs {
    #[command(subcommand)]
    pub command: ToolsCommands,
}

#[derive(Subcommand)]
pub enum ToolsCommands {
    /// List all tools
    List {
        /// Search query
        #[arg(short, long)]
        search: Option<String>,

        /// Page size
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Get tool details
    Get {
        /// Tool ID
        tool_id: String,
    },
    /// Delete a tool
    Delete {
        /// Tool ID
        tool_id: String,
    },
}
