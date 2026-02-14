use crate::cli::{RagArgs, RagCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub async fn execute(args: RagArgs, api_key: &str) -> Result<()> {
    let client = create_http_client();

    match args.command {
        RagCommands::Create { document_id, model } => {
            create_rag_index(&client, api_key, &document_id, &model).await
        }
        RagCommands::Status {
            document_id,
            rag_index_id,
        } => get_rag_status(&client, api_key, &document_id, &rag_index_id).await,
        RagCommands::Delete {
            document_id,
            rag_index_id,
        } => delete_rag_index(&client, api_key, &document_id, &rag_index_id).await,
        RagCommands::Rebuild { document_id } => rebuild_index(&client, api_key, &document_id).await,
        RagCommands::IndexStatus { document_id } => {
            get_index_status(&client, api_key, &document_id).await
        }
    }
}

#[derive(Debug, Deserialize)]
struct RagIndexResponse {
    id: String,
    model: String,
    status: String,
    #[serde(default)]
    progress_percentage: f64,
    #[serde(default)]
    document_model_index_usage: Option<RagIndexUsage>,
}

#[derive(Debug, Deserialize)]
struct RagIndexUsage {
    #[allow(dead_code)]
    used_bytes: u64,
}

async fn create_rag_index(
    client: &Client,
    api_key: &str,
    document_id: &str,
    model: &str,
) -> Result<()> {
    print_info(&format!(
        "Creating RAG index for document '{}' with model '{}'...",
        document_id.cyan(),
        model.yellow()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/knowledge-base/{}/rag-index",
        document_id
    );

    let body = json!({
        "model": model
    });

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to create RAG index")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: RagIndexResponse = response.json().await.context("Failed to parse response")?;
    print_success("RAG index creation initiated!");
    print_info(&format!("RAG Index ID: {}", result.id.yellow()));
    print_info(&format!("Model: {}", result.model));
    print_info(&format!("Status: {}", result.status));

    Ok(())
}

async fn get_rag_status(
    client: &Client,
    api_key: &str,
    document_id: &str,
    rag_index_id: &str,
) -> Result<()> {
    print_info(&format!(
        "Getting RAG index status: document='{}', index='{}'",
        document_id.cyan(),
        rag_index_id.yellow()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/knowledge-base/{}/rag-index/{}",
        document_id, rag_index_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: RagIndexResponse = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["ID", &result.id.yellow()]);
    table.add_row(vec!["Model", &result.model]);
    table.add_row(vec!["Status", &result.status]);
    table.add_row(vec![
        "Progress",
        &format!("{}%", result.progress_percentage),
    ]);
    if let Some(usage) = &result.document_model_index_usage {
        table.add_row(vec!["Index Used Bytes", &usage.used_bytes.to_string()]);
    }

    println!("{}", table);
    Ok(())
}

async fn delete_rag_index(
    client: &Client,
    api_key: &str,
    document_id: &str,
    rag_index_id: &str,
) -> Result<()> {
    print_info(&format!(
        "Deleting RAG index: document='{}', index='{}'",
        document_id.cyan(),
        rag_index_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/knowledge-base/{}/rag-index/{}",
        document_id, rag_index_id
    );

    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: RagIndexResponse = response.json().await?;
    print_success("RAG index deleted!");
    print_info(&format!("Deleted Index ID: {}", result.id.yellow()));

    Ok(())
}

#[derive(Debug, Deserialize)]
struct RagIndexStatusResponse {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    active: Option<bool>,
    #[serde(default)]
    document_id: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_message: Option<String>,
}

async fn rebuild_index(client: &Client, api_key: &str, document_id: &str) -> Result<()> {
    print_info(&format!(
        "Rebuilding RAG index for document '{}'...",
        document_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/knowledge-base/{}/rebuild-index",
        document_id
    );

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to rebuild RAG index")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: RagIndexStatusResponse =
        response.json().await.context("Failed to parse response")?;
    print_success("RAG index rebuild initiated!");

    if let Some(id) = &result.id {
        print_info(&format!("RAG Index ID: {}", id.yellow()));
    }
    if let Some(status) = &result.status {
        print_info(&format!("Status: {}", status));
    }
    if let Some(error) = &result.error {
        print_info(&format!("Error: {}", error.red()));
    }
    if let Some(error_msg) = &result.error_message {
        print_info(&format!("Error Message: {}", error_msg.red()));
    }

    Ok(())
}

async fn get_index_status(client: &Client, api_key: &str, document_id: &str) -> Result<()> {
    print_info(&format!(
        "Getting RAG index status for document '{}'...",
        document_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/knowledge-base/{}/index-status",
        document_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to get RAG index status")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: RagIndexStatusResponse =
        response.json().await.context("Failed to parse response")?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);

    if let Some(id) = &result.id {
        table.add_row(vec!["ID", &id.yellow()]);
    }
    if let Some(status) = &result.status {
        table.add_row(vec!["Status", &status]);
    }
    if let Some(active) = &result.active {
        table.add_row(vec!["Active", &active.to_string()]);
    }
    if let Some(doc_id) = &result.document_id {
        table.add_row(vec!["Document ID", &doc_id.yellow()]);
    }
    if let Some(error) = &result.error {
        table.add_row(vec!["Error", &error.red()]);
    }
    if let Some(error_msg) = &result.error_message {
        table.add_row(vec!["Error Message", &error_msg.red()]);
    }

    // If no data was returned
    if table.row_count() == 0 {
        table.add_row(vec!["Status", "No index information available"]);
    }

    println!("{}", table);
    Ok(())
}
