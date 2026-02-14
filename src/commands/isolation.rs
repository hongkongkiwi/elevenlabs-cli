use crate::cli::AudioIsolationArgs;
use crate::output::{print_info, print_success, print_warning};
use crate::utils::{confirm_overwrite, write_bytes_to_file};
use anyhow::Result;
use colored::*;
use elevenlabs_rs::{endpoints::genai::audio_isolation::AudioIsolation, ElevenLabsClient};
use std::path::Path;

pub async fn execute(args: AudioIsolationArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    let file_path = Path::new(&args.file);

    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", args.file));
    }

    // Check file extension
    if let Some(ext) = file_path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        if ![
            "mp3", "wav", "m4a", "ogg", "flac", "mp4", "mov", "avi", "webm",
        ]
        .contains(&ext.as_str())
        {
            print_warning(&format!("File format '{}' may not be supported", ext));
        }
    }

    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len();

    print_info(&format!("Isolating audio from '{}'...", args.file.cyan()));
    print_info(&format!(
        "File size: {} MB",
        (file_size as f64 / 1_048_576.0).round()
    ));

    // Create client
    let client = ElevenLabsClient::new(api_key);

    // Create endpoint - takes file path directly
    let endpoint = AudioIsolation::new(args.file.clone());

    // Process
    let start_time = std::time::Instant::now();
    let isolated_audio = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;
    let duration = start_time.elapsed();

    // Determine output path
    let output_path = if let Some(output) = args.output {
        output
    } else {
        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("audio");
        format!("{}_isolated.mp3", stem)
    };

    // Check for overwrite
    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    // Write output
    write_bytes_to_file(&isolated_audio, path)?;

    print_success(&format!(
        "Audio isolated in {:.2}s -> {}",
        duration.as_secs_f64(),
        output_path.green()
    ));

    Ok(())
}
