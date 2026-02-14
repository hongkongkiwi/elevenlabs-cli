//! Music API commands for music generation.
//!
//! This module implements the Music API for generating and managing AI music.
//! API Reference: https://elevenlabs.io/docs/api-reference/music

use crate::cli::{MusicArgs, MusicCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use crate::utils::{confirm_overwrite, write_bytes_to_file};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::path::Path;

pub async fn execute(args: MusicArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    let client = create_http_client();

    match args.command {
        MusicCommands::Generate {
            prompt,
            output,
            duration,
            influence,
        } => generate_music(&client, api_key, &prompt, output, duration, influence).await,
        MusicCommands::List { limit } => list_music(&client, api_key, limit).await,
        MusicCommands::Get { music_id } => get_music(&client, api_key, &music_id).await,
        MusicCommands::Download { music_id, output } => {
            download_music(&client, api_key, &music_id, output, assume_yes).await
        }
        MusicCommands::Delete { music_id } => delete_music(&client, api_key, &music_id).await,
    }
}

#[derive(Debug, Deserialize)]
struct MusicInfo {
    music_id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    duration_seconds: Option<f64>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MusicListResponse {
    music: Vec<MusicInfo>,
    #[allow(dead_code)]
    #[serde(default)]
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GenerateMusicResponse {
    music_id: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    audio_base64: Option<String>,
}

async fn generate_music(
    client: &Client,
    api_key: &str,
    prompt: &str,
    output: Option<String>,
    duration: Option<f32>,
    influence: Option<f32>,
) -> Result<()> {
    print_info(&format!("Generating music: \"{}\"", prompt.cyan()));

    // Validate duration
    if let Some(d) = duration {
        if !(5.0..=300.0).contains(&d) {
            return Err(anyhow::anyhow!(
                "Duration must be between 5 and 300 seconds"
            ));
        }
    }

    // Validate influence
    if let Some(i) = influence {
        if !(0.0..=1.0).contains(&i) {
            return Err(anyhow::anyhow!("Influence must be between 0.0 and 1.0"));
        }
    }

    let mut body = json!({
        "prompt": prompt
    });

    if let Some(d) = duration {
        body["duration_seconds"] = json!(d);
    }
    if let Some(i) = influence {
        body["audio_influence"] = json!(i);
    }

    let response = client
        .post("https://api.elevenlabs.io/v1/music")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to generate music")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: GenerateMusicResponse =
        response.json().await.context("Failed to parse response")?;

    print_success("Music generation started!");
    print_info(&format!("Music ID: {}", result.music_id.yellow()));

    if let Some(ref status) = result.status {
        print_info(&format!("Status: {}", status));
    }

    if let Some(ref audio) = result.audio_base64 {
        // Decode and save audio
        let audio_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, audio)
            .context("Failed to decode audio")?;

        let output_path = output.unwrap_or_else(|| format!("music_{}.mp3", result.music_id));
        let path = Path::new(&output_path);
        std::fs::write(path, &audio_bytes)?;
        print_success(&format!("Audio saved -> {}", output_path.green()));
    } else {
        print_info(
            "Audio will be available shortly. Use 'elevenlabs music download' to retrieve it.",
        );
    }

    Ok(())
}

async fn list_music(client: &Client, api_key: &str, limit: Option<u32>) -> Result<()> {
    print_info("Fetching generated music...");

    let mut url = "https://api.elevenlabs.io/v1/music".to_string();

    if let Some(l) = limit {
        url.push_str(&format!("?page_size={}", l));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch music")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let music_response: MusicListResponse =
        response.json().await.context("Failed to parse response")?;

    if music_response.music.is_empty() {
        print_info("No music found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["ID", "Title", "Duration", "Status", "Created"]);

    for music in &music_response.music {
        let title = music.title.as_deref().unwrap_or("Untitled");
        let duration = music
            .duration_seconds
            .map(|d| format!("{:.1}s", d))
            .unwrap_or_default();
        let status = music.status.as_deref().unwrap_or("ready");
        let created = music.created_at.as_deref().unwrap_or("-");

        table.add_row(vec![
            music.music_id.yellow(),
            title.cyan(),
            duration.into(),
            status.into(),
            created.into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!(
        "Found {} music tracks",
        music_response.music.len()
    ));

    Ok(())
}

async fn get_music(client: &Client, api_key: &str, music_id: &str) -> Result<()> {
    print_info(&format!("Fetching music '{}'...", music_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/music/{}", music_id);
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let music: MusicInfo = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["ID", &music.music_id.yellow()]);

    if let Some(ref title) = music.title {
        table.add_row(vec!["Title", &title.cyan()]);
    }
    if let Some(ref prompt) = music.prompt {
        table.add_row(vec!["Prompt", prompt]);
    }
    if let Some(duration) = music.duration_seconds {
        table.add_row(vec!["Duration", &format!("{:.1}s", duration)]);
    }
    if let Some(ref status) = music.status {
        table.add_row(vec!["Status", status]);
    }
    if let Some(ref created) = music.created_at {
        table.add_row(vec!["Created", created]);
    }

    println!("{}", table);
    Ok(())
}

async fn download_music(
    client: &Client,
    api_key: &str,
    music_id: &str,
    output: Option<String>,
    assume_yes: bool,
) -> Result<()> {
    print_info(&format!("Downloading music '{}'...", music_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/music/{}/audio", music_id);
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let audio = response.bytes().await?;

    let output_path = output.unwrap_or_else(|| format!("{}.mp3", music_id));

    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    write_bytes_to_file(&audio, path)?;

    print_success(&format!("Music downloaded -> {}", output_path.green()));

    Ok(())
}

async fn delete_music(client: &Client, api_key: &str, music_id: &str) -> Result<()> {
    print_info(&format!("Deleting music '{}'...", music_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/music/{}", music_id);
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!(
        "Music '{}' deleted successfully",
        music_id.green()
    ));
    Ok(())
}
