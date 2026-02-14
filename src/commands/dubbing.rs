use crate::cli::{DubbingArgs, DubbingCommands};
use crate::output::{print_error, print_info, print_success, print_warning};
use crate::utils::{confirm_overwrite, validate_file_size};
use anyhow::Result;
use colored::*;
use elevenlabs_rs::{
    endpoints::genai::dubbing::{
        DeleteDubbing, DubAVideoOrAnAudioFile, DubbingBody, GetDubbedAudio, GetDubbing,
    },
    ElevenLabsClient,
};
use std::path::Path;

pub async fn execute(args: DubbingArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    let client = ElevenLabsClient::new(api_key);

    match args.command {
        DubbingCommands::Create {
            file,
            source_lang,
            target_lang,
            num_speakers,
            watermark,
        } => {
            create_dub(
                &client,
                &file,
                &source_lang,
                &target_lang,
                num_speakers,
                watermark,
            )
            .await?
        }
        DubbingCommands::Status { dubbing_id } => get_status(&client, &dubbing_id).await?,
        DubbingCommands::Download { dubbing_id, output } => {
            download_dub(&client, &dubbing_id, output, assume_yes).await?
        }
        DubbingCommands::Delete { dubbing_id } => {
            delete_dub(&client, &dubbing_id, assume_yes).await?
        }
    }

    Ok(())
}

async fn create_dub(
    client: &ElevenLabsClient,
    file: &str,
    source_lang: &str,
    target_lang: &str,
    num_speakers: Option<u32>,
    watermark: bool,
) -> Result<()> {
    let file_path = Path::new(file);

    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file));
    }

    // Validate file size
    validate_file_size(file_path)?;

    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len();

    print_info("Creating dubbing project...");
    print_info(&format!("File: {}", file.cyan()));
    print_info(&format!("Source language: {}", source_lang.yellow()));
    print_info(&format!("Target language: {}", target_lang.yellow()));
    print_info(&format!(
        "File size: {} MB",
        (file_size as f64 / 1_048_576.0).round()
    ));

    // Build request body
    let mut body = DubbingBody::new(target_lang)
        .with_file(file)
        .with_source_lang(source_lang);

    if let Some(speakers) = num_speakers {
        body = body.with_num_speakers(speakers);
    }

    if watermark {
        body = body.with_watermark(true);
    }

    let endpoint = DubAVideoOrAnAudioFile::new(body);

    // Create dub
    let start_time = std::time::Instant::now();
    let response = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success(&format!(
        "Dubbing project created in {:.2}s",
        start_time.elapsed().as_secs_f64()
    ));
    println!("  Dubbing ID: {}", response.dubbing_id.cyan());
    println!(
        "  Expected duration: {}s",
        response.expected_duration_sec.to_string().yellow()
    );
    println!(
        "\nUse 'elevenlabs dub status {}' to check progress",
        response.dubbing_id
    );

    Ok(())
}

async fn get_status(client: &ElevenLabsClient, dubbing_id: &str) -> Result<()> {
    print_info(&format!(
        "Checking status of dubbing '{}'...",
        dubbing_id.cyan()
    ));

    let endpoint = GetDubbing::new(dubbing_id);
    let status = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("\n{}", "Dubbing Status:".bold().underline());
    println!("  ID: {}", dubbing_id.cyan());
    println!("  Name: {}", status.name);

    let colored_status = match status.status.as_str() {
        "dubbed" => status.status.green(),
        "failed" => status.status.red(),
        "dubbing" => status.status.yellow(),
        _ => status.status.normal(),
    };
    println!("  Status: {}", colored_status);

    if !status.target_languages.is_empty() {
        println!("  Target languages: {}", status.target_languages.join(", "));
    }

    if let Some(ref error) = status.error {
        print_error(&format!("Error: {}", error));
    }

    Ok(())
}

async fn download_dub(
    client: &ElevenLabsClient,
    dubbing_id: &str,
    output: Option<String>,
    assume_yes: bool,
) -> Result<()> {
    // First check status
    let status_endpoint = GetDubbing::new(dubbing_id);
    let status = client
        .hit(status_endpoint)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    if status.status != "dubbed" {
        print_warning(&format!(
            "Dubbing status: {}. The file may not be ready yet.",
            status.status
        ));

        let confirm = dialoguer::Confirm::new()
            .with_prompt("Do you want to continue?")
            .default(false)
            .interact()?;

        if !confirm {
            return Ok(());
        }
    }

    // Get target language
    let target_lang = status
        .target_languages
        .first()
        .cloned()
        .unwrap_or_else(|| "en".to_string());

    // Determine output path
    let output_path = output.unwrap_or_else(|| format!("{}_{}.mp4", dubbing_id, target_lang));

    print_info(&format!(
        "Downloading dubbed audio to '{}'...",
        output_path.cyan()
    ));

    let endpoint = GetDubbedAudio::new(dubbing_id, &target_lang);
    let audio = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    // Check for overwrite
    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    // Write file
    std::fs::write(&output_path, audio)?;

    print_success(&format!("Downloaded -> {}", output_path.green()));

    Ok(())
}

async fn delete_dub(client: &ElevenLabsClient, dubbing_id: &str, assume_yes: bool) -> Result<()> {
    print_warning(&format!(
        "You are about to delete dubbing project '{}'",
        dubbing_id
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

    let endpoint = DeleteDubbing::new(dubbing_id);
    client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success(&format!("Deleted dubbing project '{}'", dubbing_id));
    Ok(())
}
