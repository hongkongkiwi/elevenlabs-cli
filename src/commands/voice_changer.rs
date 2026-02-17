use crate::cli::VoiceChangerArgs;
use crate::output::{print_info, print_success};
use crate::utils::{
    confirm_overwrite, format_to_extension, parse_output_format, write_bytes_to_file,
};
use crate::validation::validate_voice_settings;

#[cfg(feature = "audio")]
use crate::audio::audio_io;

use anyhow::Result;
use colored::*;
use elevenlabs_rs::{
    endpoints::genai::voice_changer::{
        VoiceChanger as VoiceChangerEndpoint, VoiceChangerBody, VoiceChangerQuery,
    },
    ElevenLabsClient, Model, VoiceSettings,
};
use std::path::Path;

pub async fn execute(
    args: VoiceChangerArgs,
    api_key: &str,
    output_format: &str,
    assume_yes: bool,
) -> Result<()> {
    // Handle recording from microphone
    #[cfg(feature = "audio")]
    if args.record {
        return record_and_transform(args, api_key).await;
    }

    #[cfg(not(feature = "audio"))]
    if args.record {
        return Err(anyhow::anyhow!(
            "Recording not available. Rebuild with --features audio"
        ));
    }

    // File-based processing
    let file = match &args.file {
        Some(f) => f,
        None => {
            return Err(anyhow::anyhow!(
                "No audio file specified. Use --file or --record"
            ));
        }
    };

    let file_path = Path::new(file);

    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file));
    }

    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len();

    print_info(&format!("Transforming voice in '{}'...", file.cyan()));
    print_info(&format!(
        "File size: {} MB",
        (file_size as f64 / 1_048_576.0).round()
    ));
    print_info(&format!("Target voice: {}", args.voice.yellow()));
    print_info(&format!("Model: {}", args.model.yellow()));

    // Create client
    let client = ElevenLabsClient::new(api_key);

    // Parse model
    let model = match args.model.as_str() {
        "eleven_english_sts_v2" => Model::ElevenEnglishV2,
        "eleven_multilingual_sts_v2" => Model::ElevenMultilingualV2STS,
        _ => Model::ElevenMultilingualV2STS,
    };

    // Parse output format
    let format = parse_output_format(output_format)?;

    // Validate voice settings using validation module
    validate_voice_settings(args.stability, args.similarity_boost, args.style)?;

    // Build voice settings if any provided
    let voice_settings =
        if args.stability.is_some() || args.similarity_boost.is_some() || args.style.is_some() {
            let mut settings = VoiceSettings::default();
            if let Some(s) = args.stability {
                settings = settings.with_stability(s);
            }
            if let Some(sb) = args.similarity_boost {
                settings = settings.with_similarity_boost(sb);
            }
            if let Some(st) = args.style {
                settings = settings.with_style(st);
            }
            Some(settings)
        } else {
            None
        };

    // Build request body
    let mut body = VoiceChangerBody::new(file.clone()).with_model_id(model);

    if let Some(settings) = voice_settings {
        body = body.with_voice_settings(settings);
    }

    // Build query with output format
    let query = VoiceChangerQuery::default().with_output_format(format);

    let endpoint = VoiceChangerEndpoint::new(&args.voice, body).with_query(query);

    // Transform
    let start_time = std::time::Instant::now();
    let audio = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;
    let duration = start_time.elapsed();

    // Determine output path
    let output_path = if let Some(output) = args.output {
        output
    } else {
        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("audio");
        let ext = format_to_extension(output_format);
        format!("{}_transformed.{}", stem, ext)
    };

    // Check for overwrite
    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    // Write output
    write_bytes_to_file(&audio, path)?;

    print_success(&format!(
        "Voice transformed in {:.2}s -> {}",
        duration.as_secs_f64(),
        output_path.green()
    ));

    // Play if requested
    #[cfg(feature = "audio")]
    if args.play {
        print_info("Playing audio...");
        if let Err(e) = audio_io::play_to_speaker(&audio) {
            print_info(&format!("Could not play audio: {}", e));
        }
    }

    Ok(())
}

/// Record from microphone and transform voice
#[cfg(feature = "audio")]
async fn record_and_transform(args: VoiceChangerArgs, api_key: &str) -> Result<()> {
    use elevenlabs_rs::ElevenLabsClient;
    use std::io::Write;
    use tempfile::NamedTempFile;

    print_info(&format!(
        "Recording from microphone for {} seconds...",
        args.duration
    ));

    // Record audio
    let audio_data = match audio_io::record_from_microphone(args.duration) {
        Ok(data) => data,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to record audio: {}", e));
        }
    };

    print_info("Recording complete. Saving to temporary file...");

    // Save recorded audio to temp file
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(&audio_data)?;
    let temp_path = temp_file.path().to_str().unwrap_or("recorded_audio.wav");

    print_info(&format!("Transforming voice with '{}'...", args.voice));

    // Create client and transform
    let client = ElevenLabsClient::new(api_key);

    let model = match args.model.as_str() {
        "eleven_english_sts_v2" => Model::ElevenEnglishV2,
        _ => Model::ElevenMultilingualV2STS,
    };

    let body = VoiceChangerBody::new(temp_path).with_model_id(model);

    let endpoint =
        elevenlabs_rs::endpoints::genai::voice_changer::VoiceChanger::new(&args.voice, body);

    let audio = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success(&format!("Transformed {} bytes of audio", audio.len()));

    // Play if requested
    if args.play {
        print_info("Playing transformed audio...");
        if let Err(e) = audio_io::play_to_speaker(&audio) {
            print_info(&format!("Could not play audio: {}", e));
        }
    }

    Ok(())
}
