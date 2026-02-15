use crate::cli::{AgentArgs, AgentCommands, SpellingPatience};
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub async fn execute(args: AgentArgs, api_key: &str) -> Result<()> {
    let client = create_http_client();

    match args.command {
        AgentCommands::List { limit } => list_agents(&client, api_key, limit).await,
        AgentCommands::Summaries { limit } => list_agent_summaries(&client, api_key, limit).await,
        AgentCommands::Get { agent_id } => get_agent(&client, api_key, &agent_id).await,
        AgentCommands::Create {
            name,
            description,
            voice_id,
            first_message,
            system_prompt,
        } => {
            create_agent(
                &client,
                api_key,
                &name,
                description.as_deref(),
                voice_id.as_deref(),
                first_message.as_deref(),
                system_prompt.as_deref(),
            )
            .await
        }
        AgentCommands::Update {
            agent_id,
            name,
            description,
        } => {
            update_agent(
                &client,
                api_key,
                &agent_id,
                name.as_deref(),
                description.as_deref(),
            )
            .await
        }
        AgentCommands::Delete { agent_id } => delete_agent(&client, api_key, &agent_id).await,
        AgentCommands::Link { agent_id } => get_agent_link(&client, api_key, &agent_id).await,
        AgentCommands::Duplicate { agent_id, name } => {
            duplicate_agent(&client, api_key, &agent_id, &name).await
        }
        AgentCommands::Branches { agent_id } => {
            list_agent_branches(&client, api_key, &agent_id).await
        }
        AgentCommands::RenameBranch {
            agent_id,
            branch_id,
            name,
        } => rename_agent_branch(&client, api_key, &agent_id, &branch_id, &name).await,
        AgentCommands::BatchList { limit } => list_batch_calls(&client, api_key, limit).await,
        AgentCommands::BatchStatus { batch_id } => {
            get_batch_call_status(&client, api_key, &batch_id).await
        }
        AgentCommands::BatchDelete { batch_id } => {
            delete_batch_call(&client, api_key, &batch_id).await
        }
        AgentCommands::Simulate {
            agent_id,
            message,
            max_turns,
        } => simulate_conversation(&client, api_key, &agent_id, &message, max_turns).await,
        AgentCommands::UpdateTurn {
            agent_id,
            spelling_patience,
            silence_threshold_ms,
        } => {
            update_agent_turn_config(
                &client,
                api_key,
                &agent_id,
                spelling_patience,
                silence_threshold_ms,
            )
            .await
        }
        AgentCommands::WhatsappList => list_whatsapp_accounts(&client, api_key).await,
        AgentCommands::WidgetGet { agent_id } => {
            get_agent_widget(&client, api_key, &agent_id).await
        }
        AgentCommands::WidgetAvatar {
            agent_id,
            avatar_file,
        } => set_agent_widget_avatar(&client, api_key, &agent_id, &avatar_file).await,
    }
}

#[derive(Debug, Deserialize)]
struct AgentSummary {
    agent_id: String,
    name: String,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AgentDetail {
    agent_id: String,
    name: String,
    #[serde(default)]
    version_id: Option<String>,
    #[serde(default)]
    branch_id: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct CreateAgentResponse {
    agent_id: String,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct LinkResponse {
    url: String,
}

async fn list_agents(client: &Client, api_key: &str, limit: Option<u32>) -> Result<()> {
    print_info("Fetching agents...");

    let url = "https://api.elevenlabs.io/v1/agents";
    let mut request = client.get(url).header("xi-api-key", api_key);

    if let Some(lim) = limit {
        request = request.query(&[("limit", lim.to_string())]);
    }

    let response = request.send().await.context("Failed to fetch agents")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let agents: Vec<AgentSummary> = response.json().await.context("Failed to parse response")?;

    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Created"]);

    for agent in &agents {
        table.add_row(vec![
            agent.agent_id.yellow(),
            agent.name.cyan(),
            agent.created_at.clone().unwrap_or_default().as_str().into(),
        ]);
    }

    println!("{}", table);
    println!("\nTotal agents: {}", agents.len().to_string().green());

    Ok(())
}

async fn get_agent(client: &Client, api_key: &str, agent_id: &str) -> Result<()> {
    print_info(&format!("Fetching agent '{}'...", agent_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/agents/{}", agent_id);
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let agent: AgentDetail = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["ID", &agent.agent_id.yellow()]);
    table.add_row(vec!["Name", &agent.name.cyan()]);
    table.add_row(vec![
        "Version ID",
        agent.version_id.unwrap_or_default().as_str(),
    ]);
    table.add_row(vec![
        "Branch ID",
        agent.branch_id.unwrap_or_default().as_str(),
    ]);

    if let Some(tags) = &agent.tags {
        table.add_row(vec!["Tags", &tags.join(", ").as_str()]);
    }

    println!("{}", table);
    Ok(())
}

async fn create_agent(
    client: &Client,
    api_key: &str,
    name: &str,
    description: Option<&str>,
    voice_id: Option<&str>,
    first_message: Option<&str>,
    system_prompt: Option<&str>,
) -> Result<()> {
    print_info(&format!("Creating agent '{}'...", name.cyan()));

    let mut body = json!({
        "name": name
    });

    if let Some(desc) = description {
        body["description"] = json!(desc);
    }

    // Add conversation config with voice settings if provided
    let mut conversation_config = json!({});
    let mut has_config = false;

    if let Some(vid) = voice_id {
        conversation_config["agent"]["voice_id"] = json!(vid);
        has_config = true;
    }
    if let Some(msg) = first_message {
        conversation_config["agent"]["first_message"] = json!(msg);
        has_config = true;
    }
    if let Some(prompt) = system_prompt {
        conversation_config["agent"]["prompt"]["prompt"] = json!(prompt);
        has_config = true;
    }

    if has_config {
        body["conversation_config"] = conversation_config;
    }

    let url = "https://api.elevenlabs.io/v1/agents";
    let response = client
        .post(url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: CreateAgentResponse = response.json().await?;
    print_success("Agent created successfully!");
    print_info(&format!("Agent ID: {}", result.agent_id.yellow()));

    Ok(())
}

async fn update_agent(
    client: &Client,
    api_key: &str,
    agent_id: &str,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<()> {
    print_info(&format!("Updating agent '{}'...", agent_id.cyan()));

    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }
    if let Some(d) = description {
        body["description"] = json!(d);
    }

    let url = format!("https://api.elevenlabs.io/v1/agents/{}", agent_id);
    let response = client
        .patch(&url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success("Agent updated successfully!");
    Ok(())
}

async fn delete_agent(client: &Client, api_key: &str, agent_id: &str) -> Result<()> {
    print_info(&format!("Deleting agent '{}'...", agent_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/agents/{}", agent_id);
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Agent '{}' deleted successfully", agent_id));
    Ok(())
}

async fn get_agent_link(client: &Client, api_key: &str, agent_id: &str) -> Result<()> {
    print_info(&format!("Getting link for agent '{}'...", agent_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/agents/{}/link", agent_id);
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let link: LinkResponse = response.json().await?;
    print_success(&format!("Agent link: {}", link.url.green()));

    Ok(())
}

async fn duplicate_agent(client: &Client, api_key: &str, agent_id: &str, name: &str) -> Result<()> {
    print_info(&format!(
        "Duplicating agent '{}' as '{}'...",
        agent_id.cyan(),
        name.cyan()
    ));

    let url = format!("https://api.elevenlabs.io/v1/agents/{}/duplicate", agent_id);
    let body = json!({ "name": name });
    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: CreateAgentResponse = response.json().await?;
    print_success("Agent duplicated successfully!");
    print_info(&format!("New Agent ID: {}", result.agent_id.yellow()));

    Ok(())
}

async fn list_agent_summaries(client: &Client, api_key: &str, limit: Option<u32>) -> Result<()> {
    print_info("Fetching agent summaries...");

    let url = "https://api.elevenlabs.io/v1/convai/agents/summaries";
    let mut request = client.get(url).header("xi-api-key", api_key);

    if let Some(lim) = limit {
        request = request.query(&[("limit", lim.to_string())]);
    }

    let response = request
        .send()
        .await
        .context("Failed to fetch agent summaries")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct AgentSummaryItem {
        agent_id: String,
        name: String,
        #[serde(default)]
        description: Option<String>,
    }

    let summaries: Vec<AgentSummaryItem> =
        response.json().await.context("Failed to parse response")?;

    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Description"]);

    for summary in &summaries {
        let desc: String = summary
            .description
            .as_deref()
            .unwrap_or("-")
            .chars()
            .take(40)
            .collect();
        table.add_row(vec![
            summary.agent_id.yellow(),
            summary.name.cyan(),
            desc.as_str().into(),
        ]);
    }

    println!("{}", table);
    println!("\nTotal agents: {}", summaries.len().to_string().green());

    Ok(())
}

async fn list_agent_branches(client: &Client, api_key: &str, agent_id: &str) -> Result<()> {
    print_info(&format!(
        "Fetching branches for agent '{}'...",
        agent_id.cyan()
    ));

    let url = format!("https://api.elevenlabs.io/v1/agents/{}/branches", agent_id);
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct BranchInfo {
        branch_id: String,
        name: String,
        #[serde(default)]
        created_at: Option<String>,
    }

    let branches: Vec<BranchInfo> = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Branch ID", "Name", "Created"]);

    for branch in &branches {
        table.add_row(vec![
            branch.branch_id.yellow(),
            branch.name.cyan(),
            branch.created_at.as_deref().unwrap_or("-").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} branch(es)", branches.len()));
    Ok(())
}

async fn rename_agent_branch(
    client: &Client,
    api_key: &str,
    agent_id: &str,
    branch_id: &str,
    name: &str,
) -> Result<()> {
    print_info(&format!(
        "Renaming branch '{}' to '{}'...",
        branch_id.cyan(),
        name.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/agents/{}/branches/{}",
        agent_id, branch_id
    );
    let body = json!({ "name": name });

    let response = client
        .patch(&url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Branch renamed to '{}'", name.green()));
    Ok(())
}

async fn list_batch_calls(client: &Client, api_key: &str, limit: Option<u32>) -> Result<()> {
    print_info("Fetching batch calls...");

    let url = "https://api.elevenlabs.io/v1/convai/batch-calling";
    let mut request = client.get(url).header("xi-api-key", api_key);

    if let Some(lim) = limit {
        request = request.query(&[("limit", lim.to_string())]);
    }

    let response = request
        .send()
        .await
        .context("Failed to fetch batch calls")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct BatchCallInfo {
        batch_id: String,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        total_calls: Option<u32>,
        #[serde(default)]
        total_calls_finished: Option<u32>,
        #[serde(default)]
        created_at: Option<String>,
    }

    let batches: Vec<BatchCallInfo> = response.json().await?;

    if batches.is_empty() {
        print_info("No batch calls found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Batch ID", "Status", "Calls", "Finished", "Created"]);

    for batch in &batches {
        let total_str = batch
            .total_calls
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".to_string());
        let finished_str = batch
            .total_calls_finished
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".to_string());
        table.add_row(vec![
            batch.batch_id.yellow(),
            batch.status.as_deref().unwrap_or("-").into(),
            total_str.into(),
            finished_str.into(),
            batch.created_at.as_deref().unwrap_or("-").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} batch call(s)", batches.len()));
    Ok(())
}

async fn get_batch_call_status(client: &Client, api_key: &str, batch_id: &str) -> Result<()> {
    print_info(&format!(
        "Fetching batch call '{}' status...",
        batch_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/batch-calling/{}",
        batch_id
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

    #[derive(Deserialize)]
    struct BatchCallDetail {
        batch_id: String,
        status: String,
        #[serde(default)]
        total_calls: u32,
        #[serde(default)]
        total_calls_finished: u32,
        #[serde(default)]
        total_calls_dispatched: Option<u32>,
        #[serde(default)]
        created_at: Option<String>,
    }

    let batch: BatchCallDetail = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["Batch ID", &batch.batch_id.yellow()]);
    table.add_row(vec!["Status", &batch.status.cyan()]);
    table.add_row(vec!["Total Calls", &batch.total_calls.to_string()]);
    table.add_row(vec!["Finished", &batch.total_calls_finished.to_string()]);
    if let Some(dispatched) = batch.total_calls_dispatched {
        table.add_row(vec!["Dispatched", &dispatched.to_string()]);
    }
    if let Some(created) = &batch.created_at {
        table.add_row(vec!["Created", created]);
    }

    println!("{}", table);
    Ok(())
}

async fn delete_batch_call(client: &Client, api_key: &str, batch_id: &str) -> Result<()> {
    print_info(&format!("Deleting batch call '{}'...", batch_id.cyan()));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/batch-calling/{}",
        batch_id
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

    print_success(&format!(
        "Batch call '{}' deleted successfully",
        batch_id.green()
    ));
    Ok(())
}

async fn simulate_conversation(
    client: &Client,
    api_key: &str,
    agent_id: &str,
    message: &str,
    max_turns: u32,
) -> Result<()> {
    print_info(&format!(
        "Simulating conversation with agent '{}'...",
        agent_id.cyan()
    ));
    print_info(&format!("Initial message: \"{}\"", message.cyan()));
    print_info(&format!("Max turns: {}", max_turns));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/agents/{}/simulate",
        agent_id
    );

    let body = json!({
        "messages": [
            {
                "role": "user",
                "content": message
            }
        ],
        "max_turns": max_turns
    });

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct SimulationMessage {
        role: String,
        content: String,
    }

    #[derive(Deserialize)]
    struct SimulationResponse {
        messages: Vec<SimulationMessage>,
        #[serde(default)]
        conversation_id: Option<String>,
    }

    let result: SimulationResponse = response.json().await?;

    println!("\n{}", "Conversation Simulation:".bold().underline());

    if let Some(cid) = &result.conversation_id {
        println!("Conversation ID: {}", cid.yellow());
    }
    println!();

    for msg in &result.messages {
        match msg.role.as_str() {
            "user" => println!("{} {}", "[User]:".blue().bold(), msg.content),
            "assistant" | "agent" => println!("{} {}", "[Agent]:".green().bold(), msg.content),
            _ => println!("{}: {}", msg.role, msg.content),
        }
    }

    print_success(&format!(
        "Simulation complete ({} messages)",
        result.messages.len()
    ));
    Ok(())
}

async fn update_agent_turn_config(
    client: &Client,
    api_key: &str,
    agent_id: &str,
    spelling_patience: Option<SpellingPatience>,
    silence_threshold_ms: Option<u32>,
) -> Result<()> {
    print_info(&format!(
        "Updating turn configuration for agent '{}'...",
        agent_id.cyan()
    ));

    let mut turn_config = json!({});

    if let Some(sp) = spelling_patience {
        let patience_str = match sp {
            SpellingPatience::Auto => "auto",
            SpellingPatience::Low => "low",
            SpellingPatience::Medium => "medium",
            SpellingPatience::High => "high",
        };
        turn_config["spelling_patience"] = json!(patience_str);
        print_info(&format!(
            "Setting spelling patience to: {}",
            patience_str.cyan()
        ));
    }

    if let Some(threshold) = silence_threshold_ms {
        turn_config["silence_threshold_ms"] = json!(threshold);
        print_info(&format!(
            "Setting silence threshold to: {}ms",
            threshold.to_string().cyan()
        ));
    }

    let body = json!({
        "conversation_config": {
            "turn": turn_config
        }
    });

    let url = format!("https://api.elevenlabs.io/v1/agents/{}", agent_id);
    let response = client
        .patch(&url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success("Turn configuration updated successfully!");
    Ok(())
}

async fn list_whatsapp_accounts(client: &Client, api_key: &str) -> Result<()> {
    print_info("Fetching WhatsApp accounts...");

    // Get all agents and their WhatsApp accounts
    let url = "https://api.elevenlabs.io/v1/agents";
    let response = client
        .get(url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch agents")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct AgentWithWhatsapp {
        agent_id: String,
        name: String,
        #[serde(default)]
        whatsapp_accounts: Option<Vec<WhatsAppAccount>>,
    }

    #[derive(Deserialize)]
    struct WhatsAppAccount {
        #[allow(dead_code)]
        phone_number_id: String,
        #[serde(default)]
        phone_number: Option<String>,
        #[serde(default)]
        business_account_id: Option<String>,
    }

    let agents: Vec<AgentWithWhatsapp> =
        response.json().await.context("Failed to parse response")?;

    let mut table = Table::new();
    table.set_header(vec![
        "Agent ID",
        "Agent Name",
        "WhatsApp Phone",
        "Business Account",
    ]);

    let mut found_any = false;
    for agent in &agents {
        if let Some(accounts) = &agent.whatsapp_accounts {
            for account in accounts {
                found_any = true;
                table.add_row(vec![
                    agent.agent_id.yellow(),
                    agent.name.cyan(),
                    account.phone_number.as_deref().unwrap_or("-").into(),
                    account.business_account_id.as_deref().unwrap_or("-").into(),
                ]);
            }
        }
    }

    if !found_any {
        print_info("No WhatsApp accounts found connected to any agent");
        return Ok(());
    }

    println!("{}", table);
    print_success("WhatsApp accounts listed successfully");
    Ok(())
}

/// Get agent widget configuration
async fn get_agent_widget(client: &Client, api_key: &str, agent_id: &str) -> Result<()> {
    print_info(&format!(
        "Fetching widget configuration for agent '{}'...",
        agent_id
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/agents/{}/widget/",
        agent_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch widget configuration")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let widget: serde_json::Value = response.json().await.context("Failed to parse response")?;

    println!("\n{}", "Widget Configuration:".bold().underline());
    println!(
        "  Agent ID: {}",
        widget["agent_id"].as_str().unwrap_or("-").cyan()
    );

    if let Some(config) = widget.get("widget_config") {
        println!(
            "  Widget Enabled: {}",
            config["enabled"].as_bool().unwrap_or(false)
        );
        println!("  Position: {}", config["position"].as_str().unwrap_or("-"));

        if let Some(avatar) = config.get("avatar") {
            println!(
                "  Avatar URL: {}",
                avatar["url"].as_str().unwrap_or("-").green()
            );
        }

        if let Some(greeting) = config.get("greeting") {
            println!("  Greeting: {}", greeting.as_str().unwrap_or("-"));
        }
    }

    print_success("Widget configuration retrieved");
    Ok(())
}

/// Set agent widget avatar
async fn set_agent_widget_avatar(
    client: &Client,
    api_key: &str,
    agent_id: &str,
    avatar_file: &str,
) -> Result<()> {
    print_info(&format!(
        "Setting widget avatar for agent '{}' from '{}'...",
        agent_id, avatar_file
    ));

    let file_path = std::path::Path::new(avatar_file);
    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", avatar_file));
    }

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

    let mime = if avatar_file.ends_with(".png") {
        "image/png"
    } else if avatar_file.ends_with(".jpg") || avatar_file.ends_with(".jpeg") {
        "image/jpeg"
    } else if avatar_file.ends_with(".webp") {
        "image/webp"
    } else {
        "image/png"
    };

    let file_data = tokio::fs::read(avatar_file)
        .await
        .context("Failed to read avatar file")?;

    let part = reqwest::multipart::Part::bytes(file_data)
        .file_name(file_name.to_string())
        .mime_str(mime)
        .map_err(|e| anyhow::anyhow!("Invalid mime type: {}", e))?;

    let form = reqwest::multipart::Form::new().part("avatar_file", part);

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/agents/{}/avatar",
        agent_id
    );

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .multipart(form)
        .send()
        .await
        .context("Failed to set widget avatar")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: serde_json::Value = response.json().await.context("Failed to parse response")?;

    println!("\n{}", "Widget Avatar:".bold().underline());
    println!(
        "  Agent ID: {}",
        result["agent_id"].as_str().unwrap_or("-").cyan()
    );
    println!(
        "  Avatar URL: {}",
        result["avatar_url"].as_str().unwrap_or("-").green()
    );

    print_success("Widget avatar set successfully");
    Ok(())
}
