//! Phone Numbers API commands for managing Twilio/SIP integrations.
//!
//! This module implements the Phone Numbers API for telephony integrations.
//! API Reference: https://elevenlabs.io/docs/api-reference/phone-numbers

use crate::cli::{PhoneArgs, PhoneCommands, ProviderType};
use crate::client::create_http_client;
use crate::output::{print_info, print_success, print_warning};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub async fn execute(args: PhoneArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    let client = create_http_client();

    match args.command {
        PhoneCommands::List { agent_id } => {
            list_phone_numbers(&client, api_key, agent_id.as_deref()).await
        }
        PhoneCommands::Get { phone_id } => get_phone_number(&client, api_key, &phone_id).await,
        PhoneCommands::Import {
            number,
            provider,
            label,
            sid,
            token,
            sip_uri,
        } => {
            import_phone_number(
                &client,
                api_key,
                &number,
                provider,
                label.as_deref(),
                sid.as_deref(),
                token.as_deref(),
                sip_uri.as_deref(),
            )
            .await
        }
        PhoneCommands::Update {
            phone_id,
            label,
            agent_id,
        } => {
            update_phone_number(
                &client,
                api_key,
                &phone_id,
                label.as_deref(),
                agent_id.as_deref(),
            )
            .await
        }
        PhoneCommands::Delete { phone_id } => {
            delete_phone_number(&client, api_key, &phone_id, assume_yes).await
        }
        PhoneCommands::Test { phone_id, agent_id } => {
            test_call(&client, api_key, &phone_id, agent_id.as_deref()).await
        }
    }
}

#[derive(Debug, Deserialize)]
struct PhoneNumberInfo {
    phone_number_id: String,
    phone_number: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PhoneNumbersListResponse {
    phone_numbers: Vec<PhoneNumberInfo>,
}

#[derive(Debug, Deserialize)]
struct ImportPhoneResponse {
    phone_number_id: String,
    #[allow(dead_code)]
    phone_number: String,
}

async fn list_phone_numbers(client: &Client, api_key: &str, agent_id: Option<&str>) -> Result<()> {
    print_info("Fetching phone numbers...");

    let mut url = "https://api.elevenlabs.io/v1/convai/phone-numbers".to_string();

    if let Some(aid) = agent_id {
        url.push_str(&format!("?agent_id={}", aid));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch phone numbers")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let phones_response: PhoneNumbersListResponse =
        response.json().await.context("Failed to parse response")?;

    if phones_response.phone_numbers.is_empty() {
        print_info("No phone numbers found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["ID", "Number", "Label", "Provider", "Agent", "Status"]);

    for phone in &phones_response.phone_numbers {
        let label = phone.label.as_deref().unwrap_or("-");
        let provider = phone.provider.as_deref().unwrap_or("-");
        let agent = phone
            .agent_id
            .as_deref()
            .map(|s| {
                if s.len() > 12 {
                    format!("{}...", &s[..12])
                } else {
                    s.to_string()
                }
            })
            .unwrap_or_default();
        let status = phone.status.as_deref().unwrap_or("active");

        table.add_row(vec![
            phone.phone_number_id.yellow(),
            phone.phone_number.cyan(),
            label.into(),
            provider.into(),
            agent.into(),
            status.into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!(
        "Found {} phone number(s)",
        phones_response.phone_numbers.len()
    ));
    Ok(())
}

async fn get_phone_number(client: &Client, api_key: &str, phone_id: &str) -> Result<()> {
    print_info(&format!("Fetching phone number '{}'...", phone_id.cyan()));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/phone-numbers/{}",
        phone_id
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

    let phone: PhoneNumberInfo = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["ID", &phone.phone_number_id.yellow()]);
    table.add_row(vec!["Number", &phone.phone_number.cyan()]);

    if let Some(label) = &phone.label {
        table.add_row(vec!["Label", label]);
    }
    if let Some(provider) = &phone.provider {
        table.add_row(vec!["Provider", provider]);
    }
    if let Some(agent) = &phone.agent_id {
        table.add_row(vec!["Agent ID", agent]);
    }
    if let Some(status) = &phone.status {
        table.add_row(vec!["Status", status]);
    }
    if let Some(created) = &phone.created_at {
        table.add_row(vec!["Created", created]);
    }

    println!("{}", table);
    Ok(())
}

async fn import_phone_number(
    client: &Client,
    api_key: &str,
    number: &str,
    provider: ProviderType,
    label: Option<&str>,
    sid: Option<&str>,
    token: Option<&str>,
    sip_uri: Option<&str>,
) -> Result<()> {
    print_info(&format!("Importing phone number '{}'...", number.cyan()));

    let mut body = json!({
        "phone_number": number,
    });

    if let Some(l) = label {
        body["label"] = json!(l);
    }

    match provider {
        ProviderType::Twilio => {
            let account_sid = sid.ok_or_else(|| {
                anyhow::anyhow!("Twilio Account SID is required for Twilio provider. Use --sid")
            })?;
            let auth_token = token.ok_or_else(|| {
                anyhow::anyhow!("Twilio Auth Token is required for Twilio provider. Use --token")
            })?;

            body["provider"] = json!({
                "type": "twilio",
                "twilio_sid": account_sid,
                "twilio_token": auth_token
            });
        }
        ProviderType::Sip => {
            let uri = sip_uri.ok_or_else(|| {
                anyhow::anyhow!("SIP URI is required for SIP provider. Use --sip-uri")
            })?;

            body["provider"] = json!({
                "type": "sip_trunk",
                "sip_uri": uri
            });
        }
    }

    let response = client
        .post("https://api.elevenlabs.io/v1/convai/phone-numbers")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to import phone number")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: ImportPhoneResponse = response.json().await.context("Failed to parse response")?;
    print_success("Phone number imported successfully!");
    print_info(&format!(
        "Phone Number ID: {}",
        result.phone_number_id.yellow()
    ));

    Ok(())
}

async fn update_phone_number(
    client: &Client,
    api_key: &str,
    phone_id: &str,
    label: Option<&str>,
    agent_id: Option<&str>,
) -> Result<()> {
    print_info(&format!("Updating phone number '{}'...", phone_id.cyan()));

    let mut body = json!({});

    if let Some(l) = label {
        body["label"] = json!(l);
    }
    if let Some(aid) = agent_id {
        body["agent_id"] = json!(aid);
    }

    if body.as_object().unwrap().is_empty() {
        return Err(anyhow::anyhow!(
            "No updates specified. Use --label or --agent-id"
        ));
    }

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/phone-numbers/{}",
        phone_id
    );
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

    print_success("Phone number updated successfully!");
    Ok(())
}

async fn delete_phone_number(
    client: &Client,
    api_key: &str,
    phone_id: &str,
    assume_yes: bool,
) -> Result<()> {
    print_warning(&format!(
        "You are about to delete phone number '{}'",
        phone_id
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

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/phone-numbers/{}",
        phone_id
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
        "Phone number '{}' deleted successfully",
        phone_id.green()
    ));
    Ok(())
}

async fn test_call(
    client: &Client,
    api_key: &str,
    phone_id: &str,
    agent_id: Option<&str>,
) -> Result<()> {
    print_info(&format!(
        "Initiating test call to phone number '{}'...",
        phone_id.cyan()
    ));

    let body = json!({
        "agent_id": agent_id
    });

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/phone-numbers/{}/test-call",
        phone_id
    );

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to initiate test call")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success("Test call initiated successfully!");
    if let Some(aid) = agent_id {
        print_info(&format!("Using agent: {}", aid.yellow()));
    } else {
        print_info("Using default agent");
    }

    Ok(())
}
