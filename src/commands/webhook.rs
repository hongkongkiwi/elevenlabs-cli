use crate::cli::{WebhookArgs, WebhookCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub async fn execute(args: WebhookArgs, api_key: &str) -> Result<()> {
    let client = create_http_client();

    match args.command {
        WebhookCommands::List => list_webhooks(&client, api_key).await,
        WebhookCommands::Create { name, url, events } => {
            create_webhook(&client, api_key, &name, &url, &events).await
        }
        WebhookCommands::Delete { webhook_id } => {
            delete_webhook(&client, api_key, &webhook_id).await
        }
    }
}

#[derive(Debug, Deserialize)]
struct WebhookInfo {
    id: String,
    name: String,
    url: String,
    events: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CreateWebhookResponse {
    #[allow(dead_code)]
    id: String,
}

async fn list_webhooks(client: &Client, api_key: &str) -> Result<()> {
    print_info("Fetching webhooks...");

    let url = "https://api.elevenlabs.io/v1/webhooks";
    let response = client
        .get(url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch webhooks")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let webhooks: Vec<WebhookInfo> = response.json().await.context("Failed to parse response")?;

    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "URL", "Events"]);

    for webhook in &webhooks {
        let events = webhook.events.join(", ");
        table.add_row(vec![
            webhook.id.yellow(),
            webhook.name.cyan(),
            webhook.url.as_str().into(),
            events.as_str().into(),
        ]);
    }

    println!("{}", table);
    println!("\nTotal webhooks: {}", webhooks.len().to_string().green());

    Ok(())
}

async fn create_webhook(
    client: &Client,
    api_key: &str,
    name: &str,
    url: &str,
    events: &[String],
) -> Result<()> {
    print_info(&format!("Creating webhook '{}'...", name.cyan()));

    // Validate URL format
    let parsed_url = reqwest::Url::parse(url)
        .map_err(|_| anyhow::anyhow!("Invalid URL: '{}'. URL must be valid", url))?;
    if parsed_url.scheme() != "https" && parsed_url.scheme() != "http" {
        return Err(anyhow::anyhow!("URL must use HTTP or HTTPS scheme"));
    }

    // Validate events
    if events.is_empty() {
        return Err(anyhow::anyhow!("At least one event must be specified"));
    }

    let valid_events = [
        "model.created",
        "model.deleted",
        "voice.created",
        "voice.deleted",
        "voice.cloned",
        "agent.created",
        "agent.deleted",
        "sample.created",
        "sample.deleted",
        "history.created",
        "history.deleted",
        "dub.created",
        "dub.deleted",
        "isolation.created",
        "isolation.deleted",
        "sfx.created",
        "transcription.started",
        "transcription.completed",
    ];

    for event in events.iter() {
        if !valid_events.iter().any(|e| e == event) {
            return Err(anyhow::anyhow!(
                "Invalid event: '{}'. Valid events are: {}",
                event,
                valid_events.join(", ")
            ));
        }
    }

    let body = json!({
        "name": name,
        "url": url,
        "events": events
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/webhooks")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to create webhook")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: CreateWebhookResponse =
        response.json().await.context("Failed to parse response")?;
    print_success("Webhook created successfully!");
    print_info(&format!("Webhook ID: {}", result.id.yellow()));

    Ok(())
}

async fn delete_webhook(client: &Client, api_key: &str, webhook_id: &str) -> Result<()> {
    print_info(&format!("Deleting webhook '{}'...", webhook_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/webhooks/{}", webhook_id);
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Webhook '{}' deleted successfully", webhook_id));
    Ok(())
}
