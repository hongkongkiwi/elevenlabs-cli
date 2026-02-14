use crate::cli::{KnowledgeArgs, KnowledgeCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub async fn execute(args: KnowledgeArgs, api_key: &str) -> Result<()> {
    let client = create_http_client();

    match args.command {
        KnowledgeCommands::List { limit, offset } => {
            list_documents(&client, api_key, limit, offset).await
        }
        KnowledgeCommands::AddFromUrl {
            url,
            name,
            description,
        } => add_document_from_url(&client, api_key, &url, &name, description.as_deref()).await,
        KnowledgeCommands::AddFromText {
            text,
            name,
            description,
        } => add_document_from_text(&client, api_key, &text, &name, description.as_deref()).await,
        KnowledgeCommands::AddFromFile { file, name } => {
            add_document_from_file(&client, api_key, &file, name.as_deref()).await
        }
        KnowledgeCommands::Get { document_id } => {
            get_document(&client, api_key, &document_id).await
        }
        KnowledgeCommands::Delete { document_id } => {
            delete_document(&client, api_key, &document_id).await
        }
    }
}

async fn list_documents(
    client: &Client,
    api_key: &str,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<()> {
    print_info("Fetching knowledge base documents...");

    let url = "https://api.elevenlabs.io/v1/convai/knowledge-base";
    let mut request = client.get(url).header("xi-api-key", api_key);

    if let Some(lim) = limit {
        request = request.query(&[("limit", lim.to_string())]);
    }

    if let Some(off) = offset {
        request = request.query(&[("offset", off.to_string())]);
    }

    let response = request.send().await.context("Failed to fetch documents")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let documents: Vec<KnowledgeDocument> =
        response.json().await.context("Failed to parse response")?;

    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Created"]);

    for doc in &documents {
        let doc_type = doc.document_type.clone().unwrap_or_default();
        table.add_row(vec![
            doc.id.yellow(),
            doc.name.cyan(),
            doc_type.as_str().into(),
            doc.created_at.as_str().into(),
        ]);
    }

    println!("{}", table);
    println!("\nTotal documents: {}", documents.len().to_string().green());

    Ok(())
}

async fn add_document_from_url(
    client: &Client,
    api_key: &str,
    url: &str,
    name: &str,
    description: Option<&str>,
) -> Result<()> {
    print_info(&format!("Adding document from URL: {}", url.cyan()));

    let body = json!({
        "name": name,
        "type": "url",
        "url": url,
        "description": description.unwrap_or("")
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/convai/knowledge-base")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to add document")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: CreateDocumentResponse = response.json().await?;
    print_success("Document added successfully!");
    print_info(&format!("Document ID: {}", result.id.yellow()));

    Ok(())
}

async fn add_document_from_text(
    client: &Client,
    api_key: &str,
    text: &str,
    name: &str,
    description: Option<&str>,
) -> Result<()> {
    print_info(&format!("Adding document from text: {} bytes", text.len()));

    let body = json!({
        "name": name,
        "type": "text",
        "content": text,
        "description": description.unwrap_or("")
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/convai/knowledge-base")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to add document")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: CreateDocumentResponse = response.json().await?;
    print_success("Document added successfully!");
    print_info(&format!("Document ID: {}", result.id.yellow()));

    Ok(())
}

async fn add_document_from_file(
    client: &Client,
    api_key: &str,
    file: &str,
    name: Option<&str>,
) -> Result<()> {
    print_info(&format!("Adding document from file: {}", file.cyan()));

    // Validate file path and check if file exists
    let path = std::path::Path::new(file);
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", file));
    }

    if !path.is_file() {
        return Err(anyhow::anyhow!("Path is not a file: {}", file));
    }

    // Check file size (limit to 10MB for API)
    let metadata = std::fs::metadata(file)?;
    let file_size = metadata.len();
    if file_size > 10_000_000 {
        return Err(anyhow::anyhow!(
            "File too large ({} bytes). Maximum is 10MB.",
            file_size
        ));
    }

    let content = std::fs::read_to_string(file).context("Failed to read file")?;
    let file_name = name.unwrap_or(file);

    let body = json!({
        "name": file_name,
        "type": "text",
        "content": content
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/convai/knowledge-base")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to add document")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: CreateDocumentResponse = response.json().await?;
    print_success("Document added successfully!");
    print_info(&format!("Document ID: {}", result.id.yellow()));

    Ok(())
}

async fn get_document(client: &Client, api_key: &str, document_id: &str) -> Result<()> {
    print_info(&format!("Fetching document '{}'...", document_id.cyan()));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/knowledge-base/{}",
        document_id
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

    let doc: KnowledgeDocument = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["ID", &doc.id.yellow()]);
    table.add_row(vec!["Name", &doc.name.cyan()]);
    table.add_row(vec!["Type", &doc.document_type.clone().unwrap_or_default()]);
    table.add_row(vec!["Created", &doc.created_at]);

    println!("{}", table);
    Ok(())
}

async fn delete_document(client: &Client, api_key: &str, document_id: &str) -> Result<()> {
    print_info(&format!("Deleting document '{}'...", document_id.cyan()));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/knowledge-base/{}",
        document_id
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

    print_success(&format!("Document '{}' deleted successfully", document_id));
    Ok(())
}

#[derive(Debug, Deserialize)]
struct KnowledgeDocument {
    id: String,
    name: String,
    #[serde(default)]
    document_type: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    description: Option<String>,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateDocumentResponse {
    id: String,
}
