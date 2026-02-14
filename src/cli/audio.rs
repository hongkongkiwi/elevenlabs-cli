//! Audio processing CLI arguments (isolation, SFX, voice changer)

use clap::Args;

/// Audio isolation arguments
#[derive(Args)]
pub struct AudioIsolationArgs {
    /// Audio file to process
    #[arg(value_name = "FILE")]
    pub file: String,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,
}

/// Sound effects arguments
#[derive(Args)]
pub struct SoundEffectsArgs {
    /// Text description of the sound effect
    #[arg(value_name = "TEXT")]
    pub text: String,

    /// Duration in seconds (0.5-22, auto if not specified)
    #[arg(short, long, value_name = "SECONDS")]
    pub duration: Option<f32>,

    /// Prompt influence (0-1, default: 0.3)
    #[arg(short, long, value_name = "FLOAT")]
    pub influence: Option<f32>,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,
}

/// Voice changer arguments
#[derive(Args)]
pub struct VoiceChangerArgs {
    /// Audio file to transform
    #[arg(value_name = "FILE")]
    pub file: String,

    /// Voice ID to convert to
    #[arg(long, default_value = "Brian")]
    pub voice: String,

    /// Model to use
    #[arg(long, default_value = "eleven_multilingual_sts_v2")]
    pub model: String,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Voice settings stability
    #[arg(long, value_name = "FLOAT")]
    pub stability: Option<f32>,

    /// Voice settings similarity boost
    #[arg(long, value_name = "FLOAT")]
    pub similarity_boost: Option<f32>,

    /// Voice settings style
    #[arg(long, value_name = "FLOAT")]
    pub style: Option<f32>,
}
