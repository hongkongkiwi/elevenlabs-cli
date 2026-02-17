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
    pub file: Option<String>,

    /// Record from microphone instead of file
    #[arg(long)]
    pub record: bool,

    /// Recording duration in seconds (use with --record)
    #[arg(long, default_value = "10")]
    pub duration: f32,

    /// Voice ID to convert to
    #[arg(long, default_value = "Brian")]
    pub voice: String,

    /// Model to use
    #[arg(long, default_value = "eleven_multilingual_sts_v2")]
    pub model: String,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Play audio after processing
    #[arg(long)]
    pub play: bool,

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

/// WebSocket TTS arguments for real-time streaming
#[derive(Args)]
pub struct RealtimeTtsArgs {
    /// Text to convert to speech
    #[arg(value_name = "TEXT")]
    pub text: String,

    /// Voice ID to use
    #[arg(long, default_value = "Brian")]
    pub voice: String,

    /// Model to use
    #[arg(long, default_value = "eleven_flash_v2_5")]
    pub model: String,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Play audio in real-time as it's generated
    #[arg(long)]
    pub play: bool,

    /// Language code (auto-detected if not specified)
    #[arg(long, value_name = "CODE")]
    pub language: Option<String>,

    /// Output format
    #[arg(long, default_value = "mp3_44100_128")]
    pub output_format: String,
}
