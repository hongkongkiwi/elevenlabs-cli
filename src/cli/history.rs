//! History CLI arguments

use clap::{Args, Subcommand};

/// History arguments
#[derive(Args)]
pub struct HistoryArgs {
    #[command(subcommand)]
    pub command: HistoryCommands,
}

#[derive(Subcommand)]
pub enum HistoryCommands {
    /// List generation history
    List {
        /// Number of items to show
        #[arg(short, long, default_value = "10")]
        limit: u32,

        /// Show all details
        #[arg(short, long)]
        detailed: bool,
    },
    /// Get history item details
    Get {
        /// History item ID
        history_item_id: String,
    },
    /// Delete a history item
    Delete {
        /// History item ID
        history_item_id: String,
    },
    /// Download audio from history
    Download {
        /// History item ID
        history_item_id: String,

        /// Output file path
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,
    },
    /// Submit feedback on generated audio
    Feedback {
        /// History item ID
        history_item_id: String,

        /// Thumbs up (true) or thumbs down (false)
        #[arg(long)]
        thumbs_up: bool,

        /// Optional feedback text
        #[arg(long)]
        feedback: Option<String>,
    },
}
