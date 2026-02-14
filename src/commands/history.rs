use crate::cli::{HistoryArgs, HistoryCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success, print_warning};
use crate::utils::confirm_overwrite;
use anyhow::Result;
use colored::*;
use comfy_table::Table;
use elevenlabs_rs::{
    endpoints::admin::history::{
        DeleteHistoryItem, GetGeneratedItems, GetHistoryItem, HistoryQuery,
    },
    ElevenLabsClient,
};
use std::path::Path;

pub async fn execute(args: HistoryArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    let client = ElevenLabsClient::new(api_key);

    match args.command {
        HistoryCommands::List { limit, detailed } => list_history(&client, limit, detailed).await?,
        HistoryCommands::Get { history_item_id } => {
            get_history_item(&client, &history_item_id).await?
        }
        HistoryCommands::Delete { history_item_id } => {
            delete_history_item(&client, &history_item_id, assume_yes).await?
        }
        HistoryCommands::Download {
            history_item_id,
            output,
        } => download_history_audio(&client, api_key, &history_item_id, output, assume_yes).await?,
        HistoryCommands::Feedback {
            history_item_id,
            thumbs_up,
            feedback,
        } => submit_feedback(&client, api_key, &history_item_id, thumbs_up, feedback).await?,
    }

    Ok(())
}

async fn list_history(client: &ElevenLabsClient, limit: u32, detailed: bool) -> Result<()> {
    print_info(&format!("Fetching history (last {} items)...", limit));

    let query = HistoryQuery::default().with_page_size(limit as u16);

    let endpoint = GetGeneratedItems::with_query(query);
    let history = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    let items = history.history;

    if items.is_empty() {
        print_info("No history items found");
        return Ok(());
    }

    if detailed {
        println!("\n{}", "Generation History:".bold().underline());
        for item in &items {
            println!("\n{}", item.history_item_id.cyan());
            println!("  Voice: {}", item.voice_name.yellow());
            println!("  Voice ID: {}", item.voice_id);
            println!("  Model: {}", item.model_id.as_deref().unwrap_or("unknown"));
            println!("  Date: {}", item.date_unix);
            println!(
                "  Character count change: {}",
                item.character_count_change_from
            );
            let text_preview: String = item.text.chars().take(100).collect();
            println!("  Text: {}...", text_preview);
        }
    } else {
        let mut table = Table::new();
        table.set_header(vec!["ID", "Voice", "Date", "Chars"]);

        for item in &items {
            table.add_row(vec![
                item.history_item_id.chars().take(12).collect::<String>() + "...",
                item.voice_name.to_string(),
                item.date_unix.to_string(),
                item.character_count_change_from.to_string(),
            ]);
        }

        println!("\n{}", table);
    }

    print_success(&format!("Showing {} items", items.len()));

    Ok(())
}

async fn get_history_item(client: &ElevenLabsClient, history_item_id: &str) -> Result<()> {
    print_info(&format!(
        "Fetching history item '{}'...",
        history_item_id.cyan()
    ));

    let endpoint = GetHistoryItem::new(history_item_id);
    let item = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("\n{}", "History Item:".bold().underline());
    println!("  ID: {}", item.history_item_id.cyan());
    println!("  Voice: {}", item.voice_name.yellow());
    println!("  Voice ID: {}", item.voice_id);
    println!("  Model: {}", item.model_id.as_deref().unwrap_or("unknown"));
    println!("  Date: {}", item.date_unix);
    println!(
        "  Character count change: {}",
        item.character_count_change_from
    );
    println!("\n  Text:\n{}", item.text);

    Ok(())
}

async fn delete_history_item(
    client: &ElevenLabsClient,
    history_item_id: &str,
    assume_yes: bool,
) -> Result<()> {
    print_warning(&format!(
        "You are about to delete history item '{}'",
        history_item_id
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

    let endpoint = DeleteHistoryItem::new(history_item_id);
    client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success(&format!("Deleted history item '{}'", history_item_id));
    Ok(())
}

async fn download_history_audio(
    _client: &ElevenLabsClient,
    api_key: &str,
    history_item_id: &str,
    output: Option<String>,
    assume_yes: bool,
) -> Result<()> {
    print_info(&format!(
        "Downloading audio for history item '{}'...",
        history_item_id.cyan()
    ));

    // Determine output path
    let output_path = output.unwrap_or_else(|| format!("{}.mp3", history_item_id));

    // Check for overwrite
    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    // Use HTTP client with timeout
    let http_client = create_http_client();
    let url = format!(
        "https://api.elevenlabs.io/v1/history/{}/audio",
        history_item_id
    );
    let response = http_client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let audio = response.bytes().await?;

    // Write audio file
    std::fs::write(path, &audio)?;

    print_success(&format!("Downloaded audio -> {}", output_path.green()));
    Ok(())
}

async fn submit_feedback(
    _client: &ElevenLabsClient,
    api_key: &str,
    history_item_id: &str,
    thumbs_up: bool,
    feedback: Option<String>,
) -> Result<()> {
    print_info(&format!(
        "Submitting feedback for history item '{}'...",
        history_item_id.cyan()
    ));

    // Use HTTP client for POST request
    let http_client = create_http_client();
    let url = format!(
        "https://api.elevenlabs.io/v1/history/{}/feedback",
        history_item_id
    );

    let request = http_client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json");

    // Build the request body
    let body = if let Some(feedback_text) = feedback {
        serde_json::json!({
            "thumbs_up": thumbs_up,
            "feedback": feedback_text
        })
    } else {
        serde_json::json!({
            "thumbs_up": thumbs_up
        })
    };

    let response = request.body(body.to_string()).send().await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let feedback_type = if thumbs_up {
        "thumbs up"
    } else {
        "thumbs down"
    };
    print_success(&format!(
        "Feedback submitted: {} for '{}'",
        feedback_type, history_item_id
    ));
    Ok(())
}
