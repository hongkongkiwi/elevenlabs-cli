//! Voice management CLI arguments

use clap::{Args, Subcommand};

/// Voice management arguments
#[derive(Args)]
pub struct VoiceArgs {
    #[command(subcommand)]
    pub command: VoiceCommands,
}

#[derive(Subcommand)]
pub enum VoiceCommands {
    /// List all available voices
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    /// Get voice details
    Get {
        /// Voice ID
        voice_id: String,
    },
    /// Delete a voice
    Delete {
        /// Voice ID
        voice_id: String,
    },
    /// Clone a voice from audio samples
    Clone {
        /// Name for the new voice
        #[arg(short, long)]
        name: String,

        /// Description of the voice
        #[arg(short, long)]
        description: Option<String>,

        /// Audio files to use for cloning
        #[arg(short, long, value_name = "FILES", value_delimiter = ',')]
        samples: Vec<String>,

        /// Directory containing sample files
        #[arg(long, value_name = "DIR")]
        samples_dir: Option<String>,

        /// Labels (key=value pairs)
        #[arg(short, long, value_name = "KEY=VALUE")]
        labels: Vec<String>,
    },
    /// Get voice settings
    Settings {
        /// Voice ID
        voice_id: String,
    },
    /// Edit voice settings
    EditSettings {
        /// Voice ID
        voice_id: String,

        /// Stability (0-1)
        #[arg(long, value_name = "FLOAT")]
        stability: Option<f32>,

        /// Similarity boost (0-1)
        #[arg(long, value_name = "FLOAT")]
        similarity_boost: Option<f32>,

        /// Style (0-1)
        #[arg(long, value_name = "FLOAT")]
        style: Option<f32>,

        /// Use speaker boost
        #[arg(long)]
        speaker_boost: bool,
    },
    /// Fine-tune a voice
    FineTune {
        #[command(subcommand)]
        command: FineTuneCommands,
    },
    /// Edit voice name or description
    Edit {
        /// Voice ID
        voice_id: String,

        /// New name for the voice
        #[arg(short, long)]
        name: Option<String>,

        /// New description for the voice
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Share a voice publicly
    Share {
        /// Voice ID
        voice_id: String,
    },
    /// Find similar voices
    Similar {
        /// Voice ID to find similar voices for
        #[arg(long)]
        voice_id: Option<String>,

        /// Text description to find similar voices for
        #[arg(long)]
        text: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum FineTuneCommands {
    /// Start fine-tuning a voice
    Start {
        /// Voice ID to fine-tune
        voice_id: String,

        /// Name for the fine-tuned voice
        #[arg(short, long)]
        name: String,

        /// Description of the fine-tuned voice
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Check fine-tuning status
    Status {
        /// Voice ID
        voice_id: String,
    },
    /// Cancel fine-tuning
    Cancel {
        /// Voice ID
        voice_id: String,
    },
}
