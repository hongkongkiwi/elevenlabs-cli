//! Music CLI arguments

use clap::{Args, Subcommand};

/// Music API arguments
#[derive(Args)]
pub struct MusicArgs {
    #[command(subcommand)]
    pub command: MusicCommands,
}

#[derive(Subcommand)]
pub enum MusicCommands {
    /// Generate music from text
    Generate {
        /// Text description of the music
        #[arg(short, long)]
        prompt: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Duration in seconds (5-300)
        #[arg(short, long)]
        duration: Option<f32>,

        /// Audio influence (0-1)
        #[arg(long)]
        influence: Option<f32>,
    },
    /// List generated music
    List {
        /// Page size
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Get music details
    Get {
        /// Music ID
        music_id: String,
    },
    /// Download music audio
    Download {
        /// Music ID
        music_id: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Delete music
    Delete {
        /// Music ID
        music_id: String,
    },
}
