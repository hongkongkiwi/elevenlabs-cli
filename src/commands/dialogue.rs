use crate::cli::DialogueArgs;
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use crate::utils::{confirm_overwrite, format_to_extension, generate_output_filename};
use anyhow::{Context, Result};
use colored::*;
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub async fn execute(
    args: DialogueArgs,
    api_key: &str,
    output_format: &str,
    assume_yes: bool,
) -> Result<()> {
    if args.inputs.is_empty() {
        return Err(anyhow::anyhow!(
            "No dialogue inputs provided. Use --inputs 'text:voice_id,text:voice_id'"
        ));
    }

    // Parse inputs
    let inputs = parse_dialogue_inputs(&args.inputs)?;
    if inputs.is_empty() {
        return Err(anyhow::anyhow!(
            "No valid dialogue inputs found. Format: 'text:voice_id'"
        ));
    }

    print_info(&format!(
        "Creating dialogue with {} input(s)...",
        inputs.len().to_string().yellow()
    ));

    let client = create_http_client();

    let url = "https://api.elevenlabs.io/v1/text-to-dialogue/stream/with-timestamps";

    // Build request body
    let dialogue_inputs: Vec<_> = inputs
        .iter()
        .map(|(text, voice_id)| json!({ "text": text, "voice_id": voice_id }))
        .collect();

    let body = json!({
        "inputs": dialogue_inputs,
        "model_id": args.model,
        "output_format": output_format
    });

    // Make request
    let response = client
        .post(url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to send dialogue request")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("ElevenLabs API error: {}", error));
    }

    let dialogue_response: DialogueResponse =
        response.json().await.context("Failed to parse response")?;

    // Decode audio
    let audio_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &dialogue_response.audio_base64,
    )
    .context("Failed to decode base64 audio")?;

    // Determine output path
    let output_path = if let Some(output) = args.output {
        output
    } else {
        generate_output_filename("dialogue", format_to_extension(output_format))
    };

    // Check for overwrite
    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    // Write audio file
    let mut file = File::create(path)?;
    file.write_all(&audio_bytes)?;

    print_success(&format!("Dialogue saved -> {}", output_path.green()));

    // Print voice segments info
    if let Some(segments) = &dialogue_response.voice_segments {
        print_info(&format!(
            "Voice segments: {}",
            segments.len().to_string().yellow()
        ));
        for (i, segment) in segments.iter().enumerate() {
            println!(
                "  {}: Voice {} [{:.1}s - {:.1}s]",
                i + 1,
                segment.voice_id.yellow(),
                segment.start_time_seconds,
                segment.end_time_seconds
            );
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct DialogueResponse {
    audio_base64: String,
    #[serde(default)]
    voice_segments: Option<Vec<VoiceSegment>>,
    #[allow(dead_code)]
    #[serde(default)]
    alignment: Option<AlignmentInfo>,
}

#[derive(Debug, Deserialize)]
struct VoiceSegment {
    voice_id: String,
    start_time_seconds: f64,
    end_time_seconds: f64,
    #[allow(dead_code)]
    #[serde(default)]
    character_start_index: u32,
    #[allow(dead_code)]
    #[serde(default)]
    character_end_index: u32,
    #[allow(dead_code)]
    #[serde(default)]
    dialogue_input_index: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AlignmentInfo {
    characters: Vec<String>,
    character_start_times_seconds: Vec<f64>,
    character_end_times_seconds: Vec<f64>,
}

fn parse_dialogue_inputs(inputs: &[String]) -> Result<Vec<(String, String)>> {
    if inputs.is_empty() {
        return Err(anyhow::anyhow!(
            "No dialogue inputs provided. Use --inputs 'text:voice_id,text:voice_id'"
        ));
    }

    let mut result = Vec::new();

    for (idx, input) in inputs.iter().enumerate() {
        let parts: Vec<&str> = input.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid input format at index {}: '{}'. Expected 'text:voice_id' format. \
                Example: --inputs 'Hello:voice1,World:voice2'",
                idx,
                input
            ));
        }

        let text = parts[0].trim();
        let voice_id = parts[1].trim();

        if text.is_empty() {
            return Err(anyhow::anyhow!("Empty text in input at index {}", idx));
        }
        if voice_id.is_empty() {
            return Err(anyhow::anyhow!("Empty voice_id in input at index {}", idx));
        }

        result.push((text.to_string(), voice_id.to_string()));
    }

    Ok(result)
}
