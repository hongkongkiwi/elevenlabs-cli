use crate::cli::VoiceDesignArgs;
use crate::output::{print_info, print_success};
use crate::utils::{confirm_overwrite, write_bytes_to_file};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use colored::*;
use elevenlabs_rs::{
    endpoints::genai::text_to_voice::{TextToVoice, TextToVoiceBody},
    ElevenLabsClient,
};
use std::path::Path;

pub async fn execute(args: VoiceDesignArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    let client = ElevenLabsClient::new(api_key);

    // Validate text length
    if args.text.len() < 100 {
        return Err(anyhow::anyhow!(
            "Text must be at least 100 characters long for voice design (current: {})",
            args.text.len()
        ));
    }

    if args.text.len() > 1000 {
        return Err(anyhow::anyhow!(
            "Text must be at most 1000 characters long for voice design (current: {})",
            args.text.len()
        ));
    }

    print_info(&format!(
        "Generating voice from description: {}",
        args.description.cyan()
    ));
    print_info(&format!("Text length: {} characters", args.text.len()));

    // Build body
    let body = TextToVoiceBody::new(&args.description).with_text(&args.text);

    let endpoint = TextToVoice::new(body);

    // Generate voice previews
    let start_time = std::time::Instant::now();
    let response = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;
    let duration = start_time.elapsed();

    let previews = response.previews;
    print_success(&format!(
        "Generated {} voice previews in {:.2}s",
        previews.len(),
        duration.as_secs_f64()
    ));

    // Save previews
    for (i, preview) in previews.iter().enumerate() {
        // Decode base64 audio
        let audio = general_purpose::STANDARD
            .decode(&preview.audio_base_64)
            .map_err(|e| anyhow::anyhow!("Failed to decode audio: {}", e))?;

        let output_path = if let Some(ref output) = args.output {
            if previews.len() > 1 {
                let stem = Path::new(output)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("voice_design");
                let ext = Path::new(output)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("mp3");
                format!("{}_{}_{}.{}", stem, i, preview.generated_voice_id, ext)
            } else {
                output.clone()
            }
        } else {
            format!("voice_design_{}_{}.mp3", i, preview.generated_voice_id)
        };

        // Check for overwrite
        let path = Path::new(&output_path);

        if path.exists() && previews.len() > 1 {
            continue;
        }

        if !confirm_overwrite(path, assume_yes)? && previews.len() == 1 {
            print_info("Cancelled");
            return Ok(());
        }

        write_bytes_to_file(&audio, path)?;

        println!("  Saved preview {} -> {}", i + 1, output_path.green());
        println!("     Voice ID: {}", preview.generated_voice_id.cyan());
        println!("     Duration: {:.2}s", preview.duration_secs);
    }

    Ok(())
}
