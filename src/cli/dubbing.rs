//! Dubbing CLI arguments

use clap::{Args, Subcommand};

/// Dubbing arguments
#[derive(Args)]
pub struct DubbingArgs {
    #[command(subcommand)]
    pub command: DubbingCommands,
}

#[derive(Subcommand)]
pub enum DubbingCommands {
    /// Create a new dubbing project
    Create {
        /// File to dub
        #[arg(short, long, value_name = "FILE")]
        file: String,

        /// Source language code
        #[arg(short, long, value_name = "CODE")]
        source_lang: String,

        /// Target language code
        #[arg(short, long, value_name = "CODE")]
        target_lang: String,

        /// Number of speakers
        #[arg(long, value_name = "INT")]
        num_speakers: Option<u32>,

        /// Watermark the audio
        #[arg(long)]
        watermark: bool,
    },
    /// Get dubbing status
    Status {
        /// Dubbing ID
        dubbing_id: String,
    },
    /// Download dubbed audio
    Download {
        /// Dubbing ID
        dubbing_id: String,

        /// Output file path
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,
    },
    /// Delete a dubbing project
    Delete {
        /// Dubbing ID
        dubbing_id: String,
    },
}
