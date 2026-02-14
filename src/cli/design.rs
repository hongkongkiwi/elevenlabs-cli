//! Voice Design CLI arguments

use clap::Args;

/// Voice Design arguments
#[derive(Args)]
pub struct VoiceDesignArgs {
    /// Voice description/prompt
    #[arg(short, long)]
    pub description: String,

    /// Text to use for preview (100-1000 characters)
    #[arg(short, long)]
    pub text: String,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Output format
    #[arg(long, default_value = "mp3_44100_128")]
    pub format: String,
}
