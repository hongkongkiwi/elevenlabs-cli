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
    /// Create a new tool
    Create {
        /// Tool name
        #[arg(short, long)]
        name: String,

        /// Tool description
        #[arg(short, long)]
        description: String,

        /// Tool schema (JSON string)
        #[arg(short, long)]
        schema: String,
    },
    /// Update an existing tool
    Update {
        /// Tool ID
        tool_id: String,

        /// New tool name
        #[arg(short, long)]
        name: Option<String>,

        /// New tool description
        #[arg(short, long)]
        description: Option<String>,

        /// New tool schema (JSON string)
        #[arg(short, long)]
        schema: Option<String>,
    },
    /// Delete a tool
    Delete {
        /// Tool ID
        tool_id: String,
    },
}
