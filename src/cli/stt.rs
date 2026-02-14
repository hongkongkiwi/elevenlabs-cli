//! Speech-to-Text CLI arguments

use clap::Args;

/// Speech-to-Text arguments
#[derive(Args)]
pub struct SpeechToTextArgs {
    /// Audio file to transcribe
    #[arg(value_name = "FILE")]
    pub file: String,

    /// Model to use
    #[arg(short, long, default_value = "scribe_v1")]
    pub model: String,

    /// Language code (auto-detected if not specified)
    #[arg(short, long, value_name = "CODE")]
    pub language: Option<String>,

    /// Tag audio events
    #[arg(long, default_value = "true")]
    pub tag_audio_events: bool,

    /// Number of speakers (for diarization)
    #[arg(long, value_name = "INT")]
    pub num_speakers: Option<u32>,

    /// Timestamps granularity (none, word, character)
    #[arg(long, default_value = "word")]
    pub timestamps: String,

    /// Enable speaker diarization
    #[arg(long)]
    pub diarize: bool,

    /// Output format (json, txt, srt, vtt)
    #[arg(short, long, default_value = "txt")]
    pub format: String,

    /// Output file (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,
}
