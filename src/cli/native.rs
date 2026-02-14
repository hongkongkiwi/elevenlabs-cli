//! Audio Native CLI arguments

use clap::{Args, Subcommand};

/// Audio Native arguments
#[derive(Args)]
pub struct AudioNativeArgs {
    #[command(subcommand)]
    pub command: AudioNativeCommands,
}

#[derive(Subcommand)]
pub enum AudioNativeCommands {
    /// List all audio native projects
    List {
        /// Number of items to show
        #[arg(short, long, default_value = "10")]
        limit: u32,

        /// Page number
        #[arg(short, long, default_value = "1")]
        page: u32,
    },
    /// Get details of a specific audio native project
    Get {
        /// Project ID
        project_id: String,
    },
    /// Create a new audio native project
    Create {
        /// Project name
        #[arg(short, long)]
        name: String,

        /// Author name
        #[arg(long)]
        author: Option<String>,

        /// Title
        #[arg(long)]
        title: Option<String>,

        /// Image file path
        #[arg(long)]
        image: Option<String>,

        /// Voice ID
        #[arg(long)]
        voice_id: Option<String>,

        /// Model ID
        #[arg(long)]
        model_id: Option<String>,

        /// Content file to convert
        #[arg(long)]
        file: Option<String>,

        /// Use small player
        #[arg(long)]
        small: bool,

        /// Text color (hex)
        #[arg(long)]
        text_color: Option<String>,

        /// Background color (hex)
        #[arg(long)]
        background_color: Option<String>,

        /// Auto convert
        #[arg(long)]
        auto_convert: bool,
    },
}
