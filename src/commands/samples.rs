use crate::cli::{SamplesArgs, SamplesCommands};
use crate::output::{print_info, print_success, print_warning};
use crate::utils::{confirm_overwrite, write_bytes_to_file};
use anyhow::Result;
use colored::*;
use elevenlabs_rs::{
    endpoints::admin::samples::{DeleteSample, GetAudioFromSample},
    endpoints::admin::voice::GetVoice,
    ElevenLabsClient,
};
use std::path::Path;

pub async fn execute(args: SamplesArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    let client = ElevenLabsClient::new(api_key);

    match args.command {
        SamplesCommands::List { voice_id } => list_samples(&client, &voice_id).await?,
        SamplesCommands::Delete {
            voice_id,
            sample_id,
        } => delete_sample(&client, &voice_id, &sample_id, assume_yes).await?,
        SamplesCommands::Download {
            voice_id,
            sample_id,
            output,
        } => download_sample(&client, &voice_id, &sample_id, output, assume_yes).await?,
    }

    Ok(())
}

async fn list_samples(client: &ElevenLabsClient, voice_id: &str) -> Result<()> {
    print_info(&format!(
        "Fetching samples for voice '{}'...",
        voice_id.cyan()
    ));

    let endpoint = GetVoice::new(voice_id);
    let voice = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    if let Some(samples) = voice.samples {
        if samples.is_empty() {
            print_info("No samples found for this voice");
            return Ok(());
        }

        println!("\n{}", "Voice Samples:".bold().underline());

        for (i, sample) in samples.iter().enumerate() {
            println!("\n  Sample {}:", i + 1);

            if let Some(ref id) = sample.sample_id {
                println!("    ID: {}", id.cyan());
            }

            if let Some(ref file_name) = sample.file_name {
                println!("    File: {}", file_name);
            }

            if let Some(ref mime_type) = sample.mime_type {
                println!("    MIME Type: {}", mime_type);
            }

            if let Some(size) = sample.size_bytes {
                println!("    Size: {} KB", size / 1024);
            }
        }

        print_success(&format!("Found {} samples", samples.len()));
    } else {
        print_info("No samples found for this voice");
    }

    Ok(())
}

async fn delete_sample(
    client: &ElevenLabsClient,
    voice_id: &str,
    sample_id: &str,
    assume_yes: bool,
) -> Result<()> {
    print_warning(&format!(
        "You are about to delete sample '{}' from voice '{}'",
        sample_id, voice_id
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

    let endpoint = DeleteSample::new(voice_id, sample_id);
    client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success(&format!("Deleted sample '{}'", sample_id));
    Ok(())
}

async fn download_sample(
    client: &ElevenLabsClient,
    voice_id: &str,
    sample_id: &str,
    output: Option<String>,
    assume_yes: bool,
) -> Result<()> {
    print_info(&format!("Downloading sample '{}'...", sample_id.cyan()));

    let endpoint = GetAudioFromSample::new(voice_id, sample_id);
    let audio = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    let output_path = output.unwrap_or_else(|| format!("sample_{}.mp3", sample_id));

    // Check for overwrite
    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    write_bytes_to_file(&audio, path)?;

    print_success(&format!("Downloaded sample -> {}", output_path.green()));
    Ok(())
}
