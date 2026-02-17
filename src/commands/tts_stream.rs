use crate::cli::TtsStreamArgs;
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use crate::utils::{confirm_overwrite, generate_output_filename, write_bytes_to_file};

#[cfg(feature = "audio")]
use crate::audio::audio_io;

use anyhow::{Context, Result};
use colored::*;
use futures::StreamExt;
use std::path::Path;

pub async fn execute(args: TtsStreamArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    if args.text.is_empty() {
        return Err(anyhow::anyhow!("Text cannot be empty"));
    }

    print_info(&format!(
        "Streaming speech with timestamps using voice '{}'...",
        args.voice.cyan()
    ));
    print_info(&format!(
        "Characters: {}",
        args.text.len().to_string().yellow()
    ));

    let client = create_http_client();

    let api_url = "https://api.elevenlabs.io/v1/text-to-speech";

    // Build URL with query parameters
    let mut url = format!("{}/{}/stream/with-timestamps", api_url, args.voice);
    let mut query_params = Vec::new();

    if let Some(latency) = args.latency {
        if latency <= 4 {
            query_params.push(format!("optimize_streaming_latency={}", latency));
        }
    }

    if args.output_format != "mp3_44100_128" {
        query_params.push(format!("output_format={}", args.output_format));
    }

    if !query_params.is_empty() {
        url.push('?');
        url.push_str(&query_params.join("&"));
    }

    // Build request body with model validation
    let model = validate_model(&args.model)?;

    let mut body = serde_json::json!({
        "text": args.text,
        "model_id": model
    });

    // Add voice settings if provided
    if args.stability.is_some() || args.similarity_boost.is_some() {
        let mut settings = serde_json::json!({});

        if let Some(stability) = args.stability {
            if (0.0..=1.0).contains(&stability) {
                settings["stability"] = serde_json::json!(stability);
            } else {
                return Err(anyhow::anyhow!("Stability must be between 0.0 and 1.0"));
            }
        }

        if let Some(boost) = args.similarity_boost {
            if (0.0..=1.0).contains(&boost) {
                settings["similarity_boost"] = serde_json::json!(boost);
            } else {
                return Err(anyhow::anyhow!(
                    "Similarity boost must be between 0.0 and 1.0"
                ));
            }
        }

        body["voice_settings"] = settings;
    }

    // Make streaming request
    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to send streaming request")?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .context("Failed to read error response")?;
        return Err(anyhow::anyhow!("ElevenLabs API error: {}", error_text));
    }

    let mut stream = response.bytes_stream();
    let mut audio_chunks = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read stream chunk")?;
        audio_chunks.extend_from_slice(&chunk);
    }

    // Combine all audio
    let audio_bytes: Vec<u8> = audio_chunks;

    // Determine output path
    let output_path = if let Some(output) = args.output {
        output
    } else {
        let extension = match args.output_format.as_str() {
            f if f.starts_with("mp3_") => "mp3",
            f if f.starts_with("pcm_") => "wav",
            f if f.starts_with("ulaw_") => "ulaw",
            f if f.starts_with("opus_") => "opus",
            _ => "mp3",
        };
        generate_output_filename("speech_stream", extension)
    };

    // Check for overwrite
    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    // Write audio file
    write_bytes_to_file(&audio_bytes, path)?;

    print_success(&format!("Streamed speech saved -> {}", output_path.green()));

    // Play audio if requested
    #[cfg(feature = "audio")]
    if args.play {
        print_info("Playing audio...");
        if let Err(e) = audio_io::play_to_speaker(&audio_bytes) {
            print_info(&format!("Could not play audio: {}", e));
        }
    }

    #[cfg(not(feature = "audio"))]
    if args.play {
        print_info("Audio playback not available. Rebuild with --features audio");
    }

    Ok(())
}

fn validate_model(model: &str) -> Result<String> {
    let valid_models = [
        "eleven_multilingual_v2",
        "eleven_flash_v2_5",
        "eleven_turbo_v2",
        "eleven_turbo_v2_5",
        "eleven_v3",
    ];

    if !valid_models.contains(&model) {
        return Err(anyhow::anyhow!(
            "Invalid model: '{}'. Valid models are: {}",
            model,
            valid_models.join(", ")
        ));
    }

    Ok(model.to_string())
}
