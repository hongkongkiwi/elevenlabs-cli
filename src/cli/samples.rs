//! Samples CLI arguments

use clap::{Args, Subcommand};

/// Samples arguments
#[derive(Args)]
pub struct SamplesArgs {
    #[command(subcommand)]
    pub command: SamplesCommands,
}

#[derive(Subcommand)]
pub enum SamplesCommands {
    /// List samples for a voice
    List {
        /// Voice ID
        voice_id: String,
    },
    /// Delete a sample
    Delete {
        /// Voice ID
        voice_id: String,
        /// Sample ID
        sample_id: String,
    },
    /// Download a sample
    Download {
        /// Voice ID
        voice_id: String,
        /// Sample ID
        sample_id: String,
        /// Output file path
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,
    },
}
