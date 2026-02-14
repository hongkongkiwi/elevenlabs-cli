use crate::cli::TtsTimestampsArgs;
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use crate::utils::{
    confirm_overwrite, format_to_extension, generate_output_filename, get_input_text,
    write_bytes_to_file,
};
use anyhow::{Context, Result};
use colored::*;
use serde::Deserialize;
use std::path::Path;

pub async fn execute(
    args: TtsTimestampsArgs,
    api_key: &str,
    output_format: &str,
    assume_yes: bool,
) -> Result<()> {
    // Get input text
    let text = get_input_text(args.text, args.file)?;

    if text.is_empty() {
        return Err(anyhow::anyhow!("Text cannot be empty"));
    }

    // Create HTTP client for direct API calls (SDK doesn't have this endpoint yet)
    let client = create_http_client();

    let api_url = "https://api.elevenlabs.io/v1/text-to-speech";

    // Build URL with query params (using percent encoding for special characters)
    let mut url = format!(
        "{}/{}/with-timestamps?output_format={}",
        api_url,
        percent_encoding::utf8_percent_encode(&args.voice, percent_encoding::NON_ALPHANUMERIC),
        output_format
    );

    if let Some(latency) = args.latency {
        let latency_str = match latency {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            _ => "0",
        };
        url.push_str(&format!("&optimize_streaming_latency={}", latency_str));
    }

    let enable_logging = args.enable_logging.unwrap_or(true);
    url.push_str(&format!("&enable_logging={}", enable_logging));

    print_info(&format!(
        "Generating speech with timestamps using voice '{}'...",
        args.voice.cyan()
    ));
    print_info(&format!("Characters: {}", text.len().to_string().yellow()));

    // Make API request
    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .body(text.clone())
        .send()
        .await
        .context("Failed to send request to ElevenLabs API")?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .context("Failed to read error response")?;
        return Err(anyhow::anyhow!("ElevenLabs API error: {}", error_text));
    }

    let response_data: TtsTimestampsResponse =
        response.json().await.context("Failed to parse response")?;

    // Decode audio
    let audio_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &response_data.audio_base64,
    )
    .context("Failed to decode base64 audio")?;

    // Determine output path
    let output_path = if let Some(output) = args.output {
        output
    } else {
        generate_output_filename("speech_ts", format_to_extension(output_format))
    };

    // Check for overwrite
    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    // Write audio file
    write_bytes_to_file(&audio_bytes, path)?;

    print_success(&format!(
        "Generated speech with timestamps -> {}",
        output_path.green()
    ));

    // Generate subtitle file if requested
    if let Some(subtitles_path) = args.subtitles {
        let extension = Path::new(&subtitles_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("srt");

        match extension.to_lowercase().as_str() {
            "vtt" => write_vtt_subtitles(&response_data, &text, &subtitles_path)?,
            _ => write_srt_subtitles(&response_data, &text, &subtitles_path)?,
        }

        print_success(&format!(
            "Generated subtitles -> {}",
            subtitles_path.green()
        ));
    }

    // Print alignment info
    print_info(&format!(
        "Alignment: {} characters with timing data",
        response_data
            .alignment
            .characters
            .len()
            .to_string()
            .yellow()
    ));

    Ok(())
}

#[derive(Debug, Deserialize)]
struct TtsTimestampsResponse {
    audio_base64: String,
    alignment: AlignmentData,
    #[allow(dead_code)]
    normalized_alignment: Option<AlignmentData>,
}

#[derive(Debug, Deserialize)]
struct AlignmentData {
    characters: Vec<String>,
    character_start_times_seconds: Vec<f64>,
    character_end_times_seconds: Vec<f64>,
}

fn write_srt_subtitles(
    response: &TtsTimestampsResponse,
    _text: &str,
    output_path: &str,
) -> Result<()> {
    let mut srt_content = String::new();

    let chars = &response.alignment.characters;
    let starts = &response.alignment.character_start_times_seconds;
    let ends = &response.alignment.character_end_times_seconds;

    // Group characters into roughly subtitle-friendly chunks (about 5 chars per subtitle)
    let chunk_size = 5;
    let mut i = 0;
    let mut index = 1;

    while i < chars.len() {
        let end_idx = std::cmp::min(i + chunk_size, chars.len());
        let chunk_text: String = chars[i..end_idx].iter().map(|s| s.as_str()).collect();
        let start_time = starts[i];
        let end_time = ends[end_idx.saturating_sub(1)];

        srt_content.push_str(&format!("{}\n", index));
        srt_content.push_str(&format!(
            "{} --> {}\n",
            format_time_srt(start_time),
            format_time_srt(end_time)
        ));
        srt_content.push_str(&format!("{}\n\n", chunk_text));

        index += 1;
        i = end_idx;
    }

    std::fs::write(output_path, srt_content)?;
    Ok(())
}

fn write_vtt_subtitles(
    response: &TtsTimestampsResponse,
    _text: &str,
    output_path: &str,
) -> Result<()> {
    let mut vtt_content = String::from("WEBVTT\n\n");

    let chars = &response.alignment.characters;
    let starts = &response.alignment.character_start_times_seconds;
    let ends = &response.alignment.character_end_times_seconds;

    let chunk_size = 5;
    let mut i = 0;

    while i < chars.len() {
        let end_idx = std::cmp::min(i + chunk_size, chars.len());
        let chunk_text: String = chars[i..end_idx].iter().map(|s| s.as_str()).collect();
        let start_time = starts[i];
        let end_time = ends[end_idx.saturating_sub(1)];

        vtt_content.push_str(&format!(
            "{} --> {}\n",
            format_time_vtt(start_time),
            format_time_vtt(end_time)
        ));
        vtt_content.push_str(&format!("{}\n\n", chunk_text));

        i = end_idx;
    }

    std::fs::write(output_path, vtt_content)?;
    Ok(())
}

fn format_time_srt(seconds: f64) -> String {
    let hours = (seconds as u64) / 3600;
    let minutes = (seconds as u64 % 3600) / 60;
    let secs = seconds as u64 % 60;
    let millis = ((seconds % 1.0) * 1000.0) as u32;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

fn format_time_vtt(seconds: f64) -> String {
    let hours = (seconds as u64) / 3600;
    let minutes = (seconds as u64 % 3600) / 60;
    let secs = seconds as u64 % 60;
    let millis = ((seconds % 1.0) * 1000.0) as u32;
    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, millis)
}
