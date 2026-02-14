//! Tools API commands for agent tool management.
//!
//! This module implements the Tools API for managing agent tools.
//! API Reference: https://elevenlabs.io/docs/api-reference/tools

use crate::cli::{ToolsArgs, ToolsCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;

pub async fn execute(args: ToolsArgs, api_key: &str) -> Result<()> {
    let client = create_http_client();

    match args.command {
        ToolsCommands::List { search, limit } => {
            list_tools(&client, api_key, search.as_deref(), limit).await
        }
        ToolsCommands::Get { tool_id } => get_tool(&client, api_key, &tool_id).await,
        ToolsCommands::Delete { tool_id } => delete_tool(&client, api_key, &tool_id).await,
    }
}

#[derive(Debug, Deserialize)]
struct ToolInfo {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tool_config: Option<ToolConfig>,
    #[serde(default)]
    usage: Option<ToolUsage>,
}

#[derive(Debug, Deserialize)]
struct ToolConfig {
    #[serde(rename = "type")]
    tool_type: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    name: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ToolUsage {
    #[serde(default)]
    usage_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ToolsListResponse {
    tools: Vec<ToolInfo>,
    #[allow(dead_code)]
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    has_more: Option<bool>,
}

async fn list_tools(
    client: &Client,
    api_key: &str,
    search: Option<&str>,
    limit: Option<u32>,
) -> Result<()> {
    print_info("Fetching agent tools...");

    let mut url = "https://api.elevenlabs.io/v1/convai/tools".to_string();
    let mut params = Vec::new();

    if let Some(s) = search {
        params.push(format!(
            "search={}",
            percent_encoding::percent_encode(s.as_bytes(), percent_encoding::NON_ALPHANUMERIC)
        ));
    }
    if let Some(l) = limit {
        params.push(format!("page_size={}", l));
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch tools")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let tools_response: ToolsListResponse =
        response.json().await.context("Failed to parse response")?;

    if tools_response.tools.is_empty() {
        print_info("No tools found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Type", "Description"]);

    for tool in &tools_response.tools {
        let tool_type = tool
            .tool_config
            .as_ref()
            .and_then(|c| c.tool_type.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        let desc: String = tool
            .description
            .as_ref()
            .or(tool
                .tool_config
                .as_ref()
                .and_then(|c| c.description.as_ref()))
            .map(|s| s.chars().take(50).collect())
            .unwrap_or_default();
        table.add_row(vec![
            tool.id.yellow(),
            tool.name.cyan(),
            tool_type.into(),
            desc.as_str().into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} tools", tools_response.tools.len()));

    if tools_response.has_more.unwrap_or(false) {
        print_info("More results available. Use cursor pagination to fetch more.");
    }

    Ok(())
}

async fn get_tool(client: &Client, api_key: &str, tool_id: &str) -> Result<()> {
    print_info(&format!("Fetching tool '{}'...", tool_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/convai/tools/{}", tool_id);
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let tool: ToolInfo = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["ID", &tool.id.yellow()]);
    table.add_row(vec!["Name", &tool.name.cyan()]);

    if let Some(ref desc) = tool.description {
        table.add_row(vec!["Description", desc]);
    }
    if let Some(ref config) = tool.tool_config {
        if let Some(ref t) = config.tool_type {
            table.add_row(vec!["Type", t]);
        }
    }
    if let Some(ref usage) = tool.usage {
        if let Some(count) = usage.usage_count {
            table.add_row(vec!["Usage Count", &count.to_string()]);
        }
    }

    println!("{}", table);
    Ok(())
}

async fn delete_tool(client: &Client, api_key: &str, tool_id: &str) -> Result<()> {
    print_info(&format!("Deleting tool '{}'...", tool_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/convai/tools/{}", tool_id);
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Tool '{}' deleted successfully", tool_id.green()));
    Ok(())
}
