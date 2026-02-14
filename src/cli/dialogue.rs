//! Dialogue CLI arguments

use clap::Args;

/// Text-to-Dialogue arguments
#[derive(Args)]
pub struct DialogueArgs {
    /// Input pairs (text:voice_id)
    #[arg(short, long, value_delimiter = ',', value_name = "TEXT:VOICE_ID")]
    pub inputs: Vec<String>,

    /// Model to use
    #[arg(short, long, default_value = "eleven_v3")]
    pub model: String,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Output format
    #[arg(long, default_value = "mp3_44100_128")]
    pub output_format: String,
}
