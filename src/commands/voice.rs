use crate::cli::{FineTuneCommands, VoiceArgs, VoiceCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success, print_warning};
use crate::validation::validate_voice_settings;
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use elevenlabs_rs::{
    endpoints::admin::voice::{
        AddVoice, DeleteVoice, EditVoiceSettings, EditVoiceSettingsBody, GetVoice,
        GetVoiceSettings, GetVoices, VoiceBody,
    },
    ElevenLabsClient,
};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

pub async fn execute(args: VoiceArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    let client = ElevenLabsClient::new(api_key);
    let http_client = create_http_client();

    match args.command {
        VoiceCommands::List { detailed } => list_voices(&client, detailed).await?,
        VoiceCommands::Get { voice_id } => get_voice(&client, &voice_id).await?,
        VoiceCommands::Delete { voice_id } => delete_voice(&client, &voice_id, assume_yes).await?,
        VoiceCommands::Clone {
            name,
            description,
            samples,
            samples_dir,
            labels,
        } => {
            clone_voice(
                &client,
                &name,
                description.as_deref(),
                &samples,
                samples_dir.as_deref(),
                &labels,
            )
            .await?
        }
        VoiceCommands::Settings { voice_id } => get_settings(&client, &voice_id).await?,
        VoiceCommands::EditSettings {
            voice_id,
            stability,
            similarity_boost,
            style,
            speaker_boost,
        } => {
            edit_settings(
                &client,
                &voice_id,
                stability,
                similarity_boost,
                style,
                speaker_boost,
            )
            .await?
        }
        VoiceCommands::FineTune { command } => match command {
            FineTuneCommands::Start {
                voice_id,
                name,
                description,
            } => {
                start_fine_tune(
                    &http_client,
                    api_key,
                    &voice_id,
                    &name,
                    description.as_deref(),
                )
                .await?
            }
            FineTuneCommands::Status { voice_id } => {
                get_fine_tune_status(&http_client, api_key, &voice_id).await?
            }
            FineTuneCommands::Cancel { voice_id } => {
                cancel_fine_tune(&http_client, api_key, &voice_id, assume_yes).await?
            }
        },
        VoiceCommands::Edit {
            voice_id,
            name,
            description,
        } => {
            edit_voice(
                &http_client,
                api_key,
                &voice_id,
                name.as_deref(),
                description.as_deref(),
            )
            .await?
        }
        VoiceCommands::Share { voice_id } => share_voice(&http_client, api_key, &voice_id).await?,
        VoiceCommands::Similar { voice_id, text } => {
            find_similar_voices(&http_client, api_key, voice_id.as_deref(), text.as_deref()).await?
        }
    }

    Ok(())
}

async fn list_voices(client: &ElevenLabsClient, detailed: bool) -> Result<()> {
    print_info("Fetching voices...");

    let endpoint = GetVoices::default();
    let voices = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    if detailed {
        println!("\n{}", "Available Voices:".bold().underline());
        for voice in &voices.voices {
            println!("\n{}", voice.name.as_deref().unwrap_or("Unknown").bold());
            println!("  ID: {}", voice.voice_id.cyan());
            println!("  Category: {:?}", voice.category);
            if let Some(ref desc) = voice.description {
                println!("  Description: {}", desc);
            }
            if let Some(ref labels) = voice.labels {
                if !labels.is_empty() {
                    println!("  Labels:");
                    for (k, v) in labels {
                        println!("    {}: {}", k, v);
                    }
                }
            }
        }
    } else {
        let mut table = Table::new();
        table.set_header(vec!["Name", "ID", "Category"]);

        for voice in &voices.voices {
            table.add_row(vec![
                voice.name.clone().unwrap_or_else(|| "Unknown".to_string()),
                voice.voice_id.clone(),
                format!("{:?}", voice.category),
            ]);
        }

        println!("\n{}", table);
    }

    print_success(&format!("Found {} voices", voices.voices.len()));
    Ok(())
}

async fn get_voice(client: &ElevenLabsClient, voice_id: &str) -> Result<()> {
    print_info(&format!("Fetching voice '{}'...", voice_id.cyan()));

    let endpoint = GetVoice::new(voice_id);
    let voice = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "\n{}",
        voice
            .name
            .as_deref()
            .unwrap_or("Unknown")
            .bold()
            .underline()
    );
    println!("  ID: {}", voice.voice_id.cyan());
    println!("  Category: {:?}", voice.category);

    if let Some(ref desc) = voice.description {
        println!("  Description: {}", desc);
    }

    if let Some(ref labels) = voice.labels {
        if !labels.is_empty() {
            println!("  Labels:");
            for (k, v) in labels {
                println!("    {}: {}", k, v);
            }
        }
    }

    Ok(())
}

async fn delete_voice(client: &ElevenLabsClient, voice_id: &str, assume_yes: bool) -> Result<()> {
    print_warning(&format!("You are about to delete voice '{}'", voice_id));

    if !assume_yes {
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Are you sure?")
            .default(false)
            .interact()?;

        if !confirm {
            print_info("Cancelled");
            return Ok(());
        }
    }

    let endpoint = DeleteVoice::new(voice_id);
    client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success(&format!("Deleted voice '{}'", voice_id));
    Ok(())
}

async fn clone_voice(
    client: &ElevenLabsClient,
    name: &str,
    description: Option<&str>,
    samples: &[String],
    samples_dir: Option<&str>,
    labels: &[String],
) -> Result<()> {
    print_info(&format!("Cloning voice '{}'...", name.cyan()));

    let mut sample_files = samples.to_vec();

    if let Some(dir) = samples_dir {
        for entry in WalkDir::new(dir).max_depth(1) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if ["mp3", "wav", "m4a", "ogg", "flac"].contains(&ext.as_str()) {
                        sample_files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    if sample_files.is_empty() {
        return Err(anyhow::anyhow!("No sample files provided"));
    }

    print_info(&format!("Using {} sample files", sample_files.len()));

    // Parse labels
    let mut labels_vec = Vec::new();
    for label in labels {
        if let Some((k, v)) = label.split_once('=') {
            labels_vec.push((k.to_string(), v.to_string()));
        }
    }

    // Create body using VoiceBody::add
    let mut body = VoiceBody::add(name, sample_files);

    if let Some(desc) = description {
        body = body.with_description(desc);
    }

    if !labels_vec.is_empty() {
        body = body.with_labels(labels_vec);
    }

    let endpoint = AddVoice::new(body);

    let start_time = std::time::Instant::now();
    let response = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success(&format!(
        "Voice cloned successfully in {:.2}s!",
        start_time.elapsed().as_secs_f64()
    ));
    println!("  Voice ID: {}", response.voice_id.cyan());
    println!(
        "  Requires verification: {}",
        response.requires_verification
    );

    Ok(())
}

async fn get_settings(client: &ElevenLabsClient, voice_id: &str) -> Result<()> {
    print_info(&format!(
        "Fetching settings for voice '{}'...",
        voice_id.cyan()
    ));

    let endpoint = GetVoiceSettings::new(voice_id);
    let settings = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("\n{}", "Voice Settings:".bold().underline());
    println!("  Stability: {:?}", settings.stability);
    println!("  Similarity Boost: {:?}", settings.similarity_boost);
    println!("  Style: {:?}", settings.style);
    println!("  Speaker Boost: {:?}", settings.use_speaker_boost);
    println!("  Speed: {:?}", settings.speed);

    Ok(())
}

async fn edit_settings(
    client: &ElevenLabsClient,
    voice_id: &str,
    stability: Option<f32>,
    similarity_boost: Option<f32>,
    style: Option<f32>,
    speaker_boost: bool,
) -> Result<()> {
    print_info(&format!(
        "Updating settings for voice '{}'...",
        voice_id.cyan()
    ));

    // Validate voice settings using validation module
    validate_voice_settings(stability, similarity_boost, style)?;

    // Build new settings using EditVoiceSettingsBody
    let mut new_settings = EditVoiceSettingsBody::default();

    if let Some(s) = stability {
        new_settings = new_settings.with_stability(s);
    }
    if let Some(sb) = similarity_boost {
        new_settings = new_settings.with_similarity_boost(sb);
    }
    if let Some(st) = style {
        new_settings = new_settings.with_style(st);
    }
    new_settings = new_settings.use_speaker_boost(speaker_boost);

    let endpoint = EditVoiceSettings::new(voice_id, new_settings);
    client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success("Voice settings updated");
    Ok(())
}

/// Fine-tune response from the API
#[derive(Debug, Deserialize, Serialize)]
pub struct FineTuneResponse {
    #[serde(rename = "fineTuningId")]
    pub fine_tuning_id: String,
    #[serde(rename = "voiceId")]
    pub voice_id: String,
    #[serde(rename = "modelId")]
    pub model_id: Option<String>,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "error")]
    pub error: Option<String>,
}

/// Fine-tune status response
#[derive(Debug, Deserialize, Serialize)]
pub struct FineTuneStatusResponse {
    #[serde(rename = "fineTuningId")]
    pub fine_tuning_id: String,
    #[serde(rename = "voiceId")]
    pub voice_id: String,
    #[serde(rename = "modelId")]
    pub model_id: Option<String>,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
    #[serde(rename = "failureDetails")]
    pub failure_details: Option<Vec<FailureDetail>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FailureDetail {
    #[serde(rename = "errorCode")]
    pub error_code: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}

async fn start_fine_tune(
    client: &Client,
    api_key: &str,
    voice_id: &str,
    name: &str,
    description: Option<&str>,
) -> Result<()> {
    print_info(&format!(
        "Starting fine-tuning for voice '{}'...",
        voice_id.cyan()
    ));

    let url = format!("https://api.elevenlabs.io/v1/voices/{}/fine-tune", voice_id);

    let mut body = serde_json::json!({
        "name": name,
    });

    if let Some(desc) = description {
        body["description"] = serde_json::json!(desc);
    }

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to start fine-tuning")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: FineTuneResponse = response.json().await.context("Failed to parse response")?;

    print_success("Fine-tuning started");
    println!("  Fine-tuning ID: {}", result.fine_tuning_id.cyan());
    println!("  Voice ID: {}", result.voice_id.cyan());
    println!("  Status: {}", result.status.yellow());

    Ok(())
}

async fn get_fine_tune_status(client: &Client, api_key: &str, voice_id: &str) -> Result<()> {
    print_info(&format!(
        "Checking fine-tuning status for voice '{}'...",
        voice_id.cyan()
    ));

    let url = format!("https://api.elevenlabs.io/v1/voices/{}/fine-tune", voice_id);

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to get fine-tuning status")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: FineTuneStatusResponse =
        response.json().await.context("Failed to parse response")?;

    println!("\n{}", "Fine-tuning Status:".bold().underline());
    println!("  Fine-tuning ID: {}", result.fine_tuning_id.cyan());
    println!("  Voice ID: {}", result.voice_id.cyan());
    println!("  Status: {}", result.status.yellow());
    println!("  Created: {}", result.created_at);
    println!("  Updated: {}", result.updated_at);

    if let Some(model_id) = result.model_id {
        println!("  Model ID: {}", model_id.cyan());
    }

    if let Some(completed_at) = result.completed_at {
        println!("  Completed: {}", completed_at.green());
    }

    if let Some(failures) = result.failure_details {
        if !failures.is_empty() {
            println!("\n{}", "Failure Details:".bold().red());
            for failure in failures {
                if let Some(code) = failure.error_code {
                    println!("  Error Code: {}", code.red());
                }
                if let Some(msg) = failure.error_message {
                    println!("  Error Message: {}", msg.red());
                }
            }
        }
    }

    Ok(())
}

async fn cancel_fine_tune(
    client: &Client,
    api_key: &str,
    voice_id: &str,
    assume_yes: bool,
) -> Result<()> {
    print_warning(&format!(
        "You are about to cancel fine-tuning for voice '{}'",
        voice_id
    ));

    if !assume_yes {
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Are you sure?")
            .default(false)
            .interact()?;

        if !confirm {
            print_info("Cancelled");
            return Ok(());
        }
    }

    print_info(&format!(
        "Cancelling fine-tuning for voice '{}'...",
        voice_id.cyan()
    ));

    let url = format!("https://api.elevenlabs.io/v1/voices/{}/fine-tune", voice_id);

    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to cancel fine-tuning")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Fine-tuning cancelled for voice '{}'", voice_id));
    Ok(())
}

/// Edit voice name and/or description
async fn edit_voice(
    client: &Client,
    api_key: &str,
    voice_id: &str,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<()> {
    // Validate that at least one field is provided
    if name.is_none() && description.is_none() {
        return Err(anyhow::anyhow!(
            "At least one of --name or --description must be provided"
        ));
    }

    print_info(&format!("Editing voice '{}'...", voice_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/voices/{}", voice_id);

    let mut body = serde_json::json!({});

    if let Some(n) = name {
        body["name"] = serde_json::json!(n);
    }

    if let Some(d) = description {
        body["description"] = serde_json::json!(d);
    }

    let response = client
        .put(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to edit voice")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct EditVoiceResponse {
        voice_id: String,
        name: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        category: Option<String>,
    }

    let result: EditVoiceResponse = response.json().await.context("Failed to parse response")?;

    print_success("Voice updated successfully");
    println!("  Voice ID: {}", result.voice_id.cyan());
    println!("  Name: {}", result.name.cyan());
    if let Some(desc) = result.description {
        println!("  Description: {}", desc);
    }
    if let Some(cat) = result.category {
        println!("  Category: {}", cat);
    }

    Ok(())
}

/// Share voice publicly
async fn share_voice(client: &Client, api_key: &str, voice_id: &str) -> Result<()> {
    print_info(&format!("Sharing voice '{}' publicly...", voice_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/voices/{}/share", voice_id);

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .send()
        .await
        .context("Failed to share voice")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct ShareVoiceResponse {
        #[serde(rename = "shareUrl")]
        share_url: String,
        #[serde(rename = "voiceId")]
        voice_id: String,
    }

    let result: ShareVoiceResponse = response.json().await.context("Failed to parse response")?;

    print_success("Voice shared successfully");
    println!("  Voice ID: {}", result.voice_id.cyan());
    println!("  Share URL: {}", result.share_url.cyan());

    Ok(())
}

/// Find similar voices
async fn find_similar_voices(
    client: &Client,
    api_key: &str,
    voice_id: Option<&str>,
    text: Option<&str>,
) -> Result<()> {
    // Validate that exactly one of voice_id or text is provided
    match (voice_id, text) {
        (Some(_), Some(_)) => {
            return Err(anyhow::anyhow!(
                "Only one of --voice-id or --text can be provided, not both"
            ));
        }
        (None, None) => {
            return Err(anyhow::anyhow!(
                "At least one of --voice-id or --text must be provided"
            ));
        }
        _ => {}
    }

    print_info("Finding similar voices...");

    let url = if let Some(vid) = voice_id {
        format!(
            "https://api.elevenlabs.io/v1/voices/similar?voice_id={}",
            vid
        )
    } else if let Some(t) = text {
        let encoded = utf8_percent_encode(t, NON_ALPHANUMERIC).to_string();
        format!(
            "https://api.elevenlabs.io/v1/voices/similar?text={}",
            encoded
        )
    } else {
        unreachable!()
    };

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to find similar voices")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct SimilarVoicesResponse {
        voices: Vec<SimilarVoice>,
    }

    #[derive(Deserialize)]
    struct SimilarVoice {
        voice_id: String,
        name: String,
        #[serde(default)]
        category: Option<String>,
        #[serde(default)]
        description: Option<String>,
    }

    let result: SimilarVoicesResponse =
        response.json().await.context("Failed to parse response")?;

    if result.voices.is_empty() {
        print_info("No similar voices found");
        return Ok(());
    }

    println!("\n{}", "Similar Voices:".bold().underline());

    let mut table = Table::new();
    table.set_header(vec!["Name", "Voice ID", "Category", "Description"]);

    for voice in &result.voices {
        table.add_row(vec![
            voice.name.cyan(),
            voice.voice_id.yellow(),
            voice.category.as_deref().unwrap_or("-").into(),
            voice.description.as_deref().unwrap_or("-").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} similar voice(s)", result.voices.len()));

    Ok(())
}
