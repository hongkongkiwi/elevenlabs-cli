//! Utility functions for the ElevenLabs CLI

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Default HTTP request timeout in seconds
pub const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Maximum file size for uploads (100 MB)
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum text length for TTS (50,000 characters)
pub const MAX_TTS_TEXT_LENGTH: usize = 50_000;

/// Write bytes to a file
pub fn write_bytes_to_file(bytes: &[u8], path: &Path) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    Ok(())
}

/// Get file extension from output format
pub fn format_to_extension(format: &str) -> &str {
    if format.starts_with("mp3") {
        "mp3"
    } else if format.starts_with("pcm") || format.starts_with("wav") {
        "wav"
    } else if format.starts_with("ulaw") || format.starts_with("mulaw") {
        "ulaw"
    } else if format.starts_with("opus") {
        "opus"
    } else {
        "mp3"
    }
}

/// Parse output format string into SDK OutputFormat enum
pub fn parse_output_format(format: &str) -> Result<elevenlabs_rs::OutputFormat> {
    use elevenlabs_rs::OutputFormat;

    let fmt = match format {
        // MP3 formats
        "mp3_22050_32" => OutputFormat::Mp3_22050Hz32kbps,
        "mp3_44100_32" => OutputFormat::Mp3_44100Hz32kbps,
        "mp3_44100_64" => OutputFormat::Mp3_44100Hz64kbps,
        "mp3_44100_96" => OutputFormat::Mp3_44100Hz96kbps,
        "mp3_44100_128" => OutputFormat::Mp3_44100Hz128kbps,
        "mp3_44100_192" => OutputFormat::Mp3_44100Hz192kbps,

        // PCM formats
        "pcm_8000" => OutputFormat::Pcm8000Hz,
        "pcm_16000" => OutputFormat::Pcm16000Hz,
        "pcm_22050" => OutputFormat::Pcm22050Hz,
        "pcm_24000" => OutputFormat::Pcm24000Hz,
        "pcm_44100" => OutputFormat::Pcm44100Hz,

        // MuLaw formats
        "ulaw_8000" | "mulaw_8000" => OutputFormat::MuLaw8000Hz,

        // OPUS formats
        "opus_48000_32" => OutputFormat::Opus48000Hz32kbps,
        "opus_48000_64" => OutputFormat::Opus48000Hz64kbps,
        "opus_48000_96" => OutputFormat::Opus48000Hz96kbps,
        "opus_48000_128" => OutputFormat::Opus48000Hz128kbps,
        "opus_48000_192" => OutputFormat::Opus48000Hz192kbps,

        // WAV formats (2026 API) - mapped to equivalent PCM
        "wav_8000" => OutputFormat::Pcm8000Hz,
        "wav_16000" => OutputFormat::Pcm16000Hz,
        "wav_22050" => OutputFormat::Pcm22050Hz,
        "wav_24000" => OutputFormat::Pcm24000Hz,
        "wav_44100" => OutputFormat::Pcm44100Hz,

        _ => OutputFormat::Mp3_44100Hz128kbps,
    };
    Ok(fmt)
}

/// Read text from file or return the text directly
pub fn get_input_text(text: Option<String>, file: Option<String>) -> Result<String> {
    match (text, file) {
        (Some(t), _) => Ok(t),
        (_, Some(f)) => {
            let content = std::fs::read_to_string(&f)?;
            Ok(content)
        }
        (None, None) => Err(anyhow::anyhow!("Either text or file must be provided")),
    }
}

/// Generate default output filename
pub fn generate_output_filename(prefix: &str, extension: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    format!("{}_{}.{}", prefix, timestamp, extension)
}

/// Check if file exists and confirm overwrite
pub fn confirm_overwrite(path: &Path, assume_yes: bool) -> Result<bool> {
    if !path.exists() {
        return Ok(true);
    }

    if assume_yes {
        return Ok(true);
    }

    crate::output::print_warning(&format!("File '{}' already exists.", path.display()));

    if let Ok(confirm) = dialoguer::Confirm::new()
        .with_prompt("Do you want to overwrite?")
        .default(false)
        .interact()
    {
        Ok(confirm)
    } else {
        Ok(false)
    }
}

/// Validate file size is within limits
pub fn validate_file_size(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    let size = metadata.len();

    if size == 0 {
        return Err(anyhow::anyhow!("File is empty: {}", path.display()));
    }

    if size > MAX_FILE_SIZE {
        return Err(anyhow::anyhow!(
            "File too large: {} bytes (max: {} bytes)",
            size,
            MAX_FILE_SIZE
        ));
    }

    Ok(())
}

/// Validate text length is within TTS limits
pub fn validate_text_length(text: &str) -> Result<()> {
    let len = text.len();

    if len == 0 {
        return Err(anyhow::anyhow!("Text cannot be empty"));
    }

    if len > MAX_TTS_TEXT_LENGTH {
        return Err(anyhow::anyhow!(
            "Text too long: {} characters (max: {} characters)",
            len,
            MAX_TTS_TEXT_LENGTH
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_format_to_extension() {
        assert_eq!(format_to_extension("mp3_44100_128"), "mp3");
        assert_eq!(format_to_extension("mp3_44100_192"), "mp3");
        assert_eq!(format_to_extension("pcm_16000"), "wav");
        assert_eq!(format_to_extension("pcm_44100"), "wav");
        assert_eq!(format_to_extension("wav_8000"), "wav");
        assert_eq!(format_to_extension("wav_44100"), "wav");
        assert_eq!(format_to_extension("ulaw_8000"), "ulaw");
        assert_eq!(format_to_extension("opus_48000_128"), "opus");
        assert_eq!(format_to_extension("unknown"), "mp3");
    }

    #[test]
    fn test_parse_output_format() {
        assert!(parse_output_format("mp3_44100_128").is_ok());
        assert!(parse_output_format("mp3_44100_192").is_ok());
        assert!(parse_output_format("pcm_16000").is_ok());
        assert!(parse_output_format("pcm_44100").is_ok());
        assert!(parse_output_format("wav_8000").is_ok());
        assert!(parse_output_format("wav_44100").is_ok());
        assert!(parse_output_format("opus_48000_128").is_ok());
        assert!(parse_output_format("unknown_format").is_ok());
    }

    #[test]
    fn test_write_bytes_to_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.bin");
        let data = b"Hello, World!";

        write_bytes_to_file(data, &file_path).unwrap();

        let contents = std::fs::read(&file_path).unwrap();
        assert_eq!(contents, data);
    }

    #[test]
    fn test_generate_output_filename() {
        let filename = generate_output_filename("speech", "mp3");
        assert!(filename.starts_with("speech_"));
        assert!(filename.ends_with(".mp3"));
    }

    #[test]
    fn test_get_input_text() {
        let result = get_input_text(Some("Hello".to_string()), None).unwrap();
        assert_eq!(result, "Hello");
        assert!(get_input_text(None, None).is_err());
    }

    #[test]
    fn test_validate_text_length() {
        assert!(validate_text_length("Hello").is_ok());
        assert!(validate_text_length("").is_err());
        let long_text = "x".repeat(60_000);
        assert!(validate_text_length(&long_text).is_err());
    }
}
