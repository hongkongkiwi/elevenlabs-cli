use crate::cli::SpeechToTextArgs;
use crate::output::{print_info, print_success};
use crate::utils::validate_file_size;
use anyhow::Result;
use colored::*;
use elevenlabs_rs::{
    endpoints::genai::speech_to_text::{
        CreateTranscript, CreateTranscriptBody, Granularity, SpeechToTextModel,
    },
    ElevenLabsClient,
};
use serde::Serialize;
use std::fs;
use std::path::Path;

/// JSON output structure for transcription results
#[derive(Debug, Serialize)]
struct TranscriptionJsonOutput {
    text: String,
    language_code: String,
    language_probability: f64,
    words: Vec<WordInfo>,
}

#[derive(Debug, Serialize)]
struct WordInfo {
    text: String,
    start: Option<f64>,
    end: Option<f64>,
    speaker_id: Option<String>,
}

pub async fn execute(args: SpeechToTextArgs, api_key: &str) -> Result<()> {
    let file_path = Path::new(&args.file);

    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", args.file));
    }

    // Validate file size using utility
    validate_file_size(file_path)?;

    let metadata = fs::metadata(file_path)?;
    let file_size = metadata.len();

    print_info(&format!("Transcribing '{}'...", args.file.cyan()));
    print_info(&format!(
        "File size: {} MB",
        (file_size as f64 / 1_048_576.0).round()
    ));
    print_info(&format!("Model: {}", args.model.yellow()));

    // Create client
    let client = ElevenLabsClient::new(api_key);

    // Parse model
    let model = match args.model.as_str() {
        "scribe_v1" => SpeechToTextModel::ScribeV1,
        "scribe_v1_base" => SpeechToTextModel::ScribeV1Base,
        _ => SpeechToTextModel::ScribeV1,
    };

    // Build request body
    let mut body =
        CreateTranscriptBody::new(model, &args.file).with_tag_audio_events(args.tag_audio_events);

    if let Some(lang) = &args.language {
        body = body.with_language_code(lang);
    }

    if let Some(speakers) = args.num_speakers {
        body = body.with_num_speakers(speakers);
    }

    // Parse timestamps granularity
    let timestamps = match args.timestamps.as_str() {
        "none" => Granularity::None,
        "character" => Granularity::Character,
        _ => Granularity::Word,
    };
    body = body.with_timestamps_granularity(timestamps);

    if args.diarize {
        body = body.with_diarize(true);
    }

    // Create endpoint
    let endpoint = CreateTranscript::new(body);

    // Transcribe
    let start_time = std::time::Instant::now();
    let result = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;
    let duration = start_time.elapsed();

    // Format output based on requested format
    let output = format_output(&result, &args.format, args.diarize)?;

    // Write output
    if let Some(output_file) = &args.output {
        fs::write(output_file, output)?;
        print_success(&format!(
            "Transcription complete in {:.2}s -> {}",
            duration.as_secs_f64(),
            output_file.green()
        ));
    } else {
        println!("\n{}", "Transcription:".bold().underline());
        println!("{}", output);
        print_success(&format!("Completed in {:.2}s", duration.as_secs_f64()));
    }

    // Print metadata
    print_info(&format!(
        "Detected language: {} ({:.1}% confidence)",
        result.language_code.cyan(),
        result.language_probability * 100.0
    ));

    Ok(())
}

fn format_output(
    result: &elevenlabs_rs::endpoints::genai::speech_to_text::CreateTranscriptResponse,
    format: &str,
    diarize: bool,
) -> Result<String> {
    // Get words from result - convert types as needed
    let words: Vec<WordInfo> = result
        .words
        .iter()
        .map(|w| WordInfo {
            text: w.text.clone(),
            start: w.start.map(|f| f as f64),
            end: w.end.map(|f| f as f64),
            speaker_id: w.speaker_id.clone(),
        })
        .collect();

    match format {
        "json" => {
            // Reconstruct full text from words
            let full_text: String = result
                .words
                .iter()
                .map(|w| w.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            let output = TranscriptionJsonOutput {
                text: full_text,
                language_code: result.language_code.clone(),
                language_probability: result.language_probability as f64,
                words,
            };

            Ok(serde_json::to_string_pretty(&output)?)
        }
        "srt" => format_srt(&result.words, diarize),
        "vtt" => format_vtt(&result.words, diarize),
        _ => {
            // Plain text - reconstruct from words
            let full_text: String = result
                .words
                .iter()
                .map(|w| w.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            Ok(full_text)
        }
    }
}

fn format_srt(
    words: &[elevenlabs_rs::endpoints::genai::speech_to_text::Word],
    diarize: bool,
) -> Result<String> {
    let mut srt = String::new();
    let mut counter = 1;

    for word in words {
        let start = match word.start {
            Some(s) => s,
            None => continue,
        };
        let end = match word.end {
            Some(e) => e,
            None => continue,
        };

        let start_secs = (start / 1000.0) as u64;
        let start_ms = (start % 1000.0) as u64;
        let end_secs = (end / 1000.0) as u64;
        let end_ms = (end % 1000.0) as u64;

        srt.push_str(&format!("{}\n", counter));
        srt.push_str(&format!(
            "{:02}:{:02}:{:02},{:03} --> {:02}:{:02}:{:02},{:03}\n",
            start_secs / 3600,
            (start_secs % 3600) / 60,
            start_secs % 60,
            start_ms,
            end_secs / 3600,
            (end_secs % 3600) / 60,
            end_secs % 60,
            end_ms
        ));

        if diarize {
            if let Some(ref speaker) = word.speaker_id {
                srt.push_str(&format!("[{}] ", speaker));
            }
        }

        srt.push_str(&format!("{}\n\n", word.text));
        counter += 1;
    }

    Ok(srt)
}

fn format_vtt(
    words: &[elevenlabs_rs::endpoints::genai::speech_to_text::Word],
    diarize: bool,
) -> Result<String> {
    let mut vtt = String::from("WEBVTT\n\n");

    for word in words {
        let start = match word.start {
            Some(s) => s,
            None => continue,
        };
        let end = match word.end {
            Some(e) => e,
            None => continue,
        };

        let start_secs = (start / 1000.0) as u64;
        let start_ms = (start % 1000.0) as u64;
        let end_secs = (end / 1000.0) as u64;
        let end_ms = (end % 1000.0) as u64;

        vtt.push_str(&format!(
            "{:02}:{:02}:{:02}.{:03} --> {:02}:{:02}:{:02}.{:03}\n",
            start_secs / 3600,
            (start_secs % 3600) / 60,
            start_secs % 60,
            start_ms,
            end_secs / 3600,
            (end_secs % 3600) / 60,
            end_secs % 60,
            end_ms
        ));

        if diarize {
            if let Some(ref speaker) = word.speaker_id {
                vtt.push_str(&format!("<v {}>", speaker));
            }
        }

        vtt.push_str(&format!("{}\n\n", word.text));
    }

    Ok(vtt)
}
