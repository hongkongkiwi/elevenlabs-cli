use crate::cli::SoundEffectsArgs;
use crate::output::{print_info, print_success};
use crate::utils::{confirm_overwrite, generate_output_filename, write_bytes_to_file};
use anyhow::Result;
use colored::*;
use elevenlabs_rs::{
    endpoints::genai::sound_effects::{CreateSoundEffect, CreateSoundEffectBody},
    ElevenLabsClient,
};
use std::path::Path;

pub async fn execute(args: SoundEffectsArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    if args.text.is_empty() {
        return Err(anyhow::anyhow!("Text description cannot be empty"));
    }

    // Validate duration if provided
    if let Some(duration) = args.duration {
        if !(0.5..=22.0).contains(&duration) {
            return Err(anyhow::anyhow!(
                "Duration must be between 0.5 and 22 seconds"
            ));
        }
    }

    // Validate influence if provided
    if let Some(influence) = args.influence {
        if !(0.0..=1.0).contains(&influence) {
            return Err(anyhow::anyhow!("Influence must be between 0 and 1"));
        }
    }

    print_info("Generating sound effect...");
    print_info(&format!("Prompt: {}", args.text.cyan()));

    if let Some(duration) = args.duration {
        print_info(&format!("Duration: {}s", duration));
    } else {
        print_info("Duration: auto");
    }

    // Create client
    let client = ElevenLabsClient::new(api_key);

    // Build request body
    let mut body = CreateSoundEffectBody::new(&args.text);

    if let Some(duration) = args.duration {
        body = body.with_duration_seconds(duration);
    }
    if let Some(influence) = args.influence {
        body = body.with_prompt_influence(influence);
    }

    let endpoint = CreateSoundEffect::new(body);

    // Generate
    let start_time = std::time::Instant::now();
    let audio = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;
    let duration = start_time.elapsed();

    // Determine output path
    let output_path = if let Some(output) = args.output {
        output
    } else {
        generate_output_filename("sound_effect", "mp3")
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
        "Sound effect generated in {:.2}s -> {}",
        duration.as_secs_f64(),
        output_path.green()
    ));

    Ok(())
}
