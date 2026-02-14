use crate::cli::{PronunciationArgs, PronunciationCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success, print_warning};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use elevenlabs_rs::{
    endpoints::admin::pronunciation::{CreateDictionary, CreateDictionaryBody, GetDictionaries},
    ElevenLabsClient,
};
use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub async fn execute(args: PronunciationArgs, api_key: &str) -> Result<()> {
    let client = ElevenLabsClient::new(api_key);
    let http_client = create_http_client();

    match args.command {
        PronunciationCommands::List => list_dictionaries(&client).await?,
        PronunciationCommands::Add {
            file,
            name,
            description,
        } => add_dictionary(&client, &file, &name, description).await?,
        PronunciationCommands::Delete { dictionary_id } => {
            delete_dictionary(&http_client, api_key, &dictionary_id).await?
        }
        PronunciationCommands::Rules { dictionary_id } => {
            list_rules(&http_client, api_key, &dictionary_id).await?
        }
        PronunciationCommands::AddRules {
            dictionary_id,
            rules_file,
        } => add_rules(&http_client, api_key, &dictionary_id, &rules_file).await?,
        PronunciationCommands::RemoveRules {
            dictionary_id,
            rules_file,
        } => remove_rules(&http_client, api_key, &dictionary_id, &rules_file).await?,
        PronunciationCommands::GetPls {
            dictionary_id,
            output,
        } => get_pls_file(&http_client, api_key, &dictionary_id, &output).await?,
    }

    Ok(())
}

async fn list_dictionaries(client: &ElevenLabsClient) -> Result<()> {
    print_info("Fetching pronunciation dictionaries...");

    let endpoint = GetDictionaries::default();
    let response = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    if response.pronunciation_dictionaries.is_empty() {
        print_info("No pronunciation dictionaries found");
        return Ok(());
    }

    println!("\n{}", "Pronunciation Dictionaries:".bold().underline());

    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Version", "Created"]);

    for dict in &response.pronunciation_dictionaries {
        table.add_row(vec![
            dict.id.chars().take(12).collect::<String>() + "...",
            dict.name.clone(),
            dict.latest_version_id.chars().take(8).collect::<String>() + "...",
            dict.creation_time_unix.to_string(),
        ]);
    }

    println!("{}", table);
    print_success(&format!(
        "Found {} dictionaries",
        response.pronunciation_dictionaries.len()
    ));

    Ok(())
}

async fn add_dictionary(
    client: &ElevenLabsClient,
    file: &str,
    name: &str,
    description: Option<String>,
) -> Result<()> {
    let file_path = Path::new(file);

    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file));
    }

    if let Some(ext) = file_path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        if ext != "pls" && ext != "xml" {
            print_warning("File should be a .pls (Pronunciation Lexicon Specification) file");
        }
    }

    print_info(&format!(
        "Adding pronunciation dictionary '{}'...",
        name.cyan()
    ));

    let mut body = CreateDictionaryBody::new(file, name);

    if let Some(desc) = description {
        body = body.with_description(&desc);
    }

    let endpoint = CreateDictionary::new(body);
    let response = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success(&format!("Added dictionary '{}'", name));
    println!("  Dictionary ID: {}", response.id.cyan());
    println!("  Version ID: {}", response.version_id);

    Ok(())
}

async fn delete_dictionary(client: &Client, api_key: &str, dictionary_id: &str) -> Result<()> {
    print_info(&format!(
        "Deleting pronunciation dictionary '{}'...",
        dictionary_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation-dictionaries/{}",
        dictionary_id
    );
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to delete dictionary")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Deleted dictionary '{}'", dictionary_id.green()));
    Ok(())
}

async fn list_rules(client: &Client, api_key: &str, dictionary_id: &str) -> Result<()> {
    print_info(&format!(
        "Fetching rules for dictionary '{}'...",
        dictionary_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation/dictionaries/{}/rules",
        dictionary_id
    );
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch rules")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let rules: serde_json::Value = response.json().await.context("Failed to parse rules")?;

    if rules.is_array() && rules.as_array().unwrap().is_empty() {
        print_info("No rules found in this dictionary");
        return Ok(());
    }

    println!("\n{}", "Pronunciation Rules:".bold().underline());
    println!("{}", serde_json::to_string_pretty(&rules).unwrap());

    print_success(&format!(
        "Found rules in dictionary '{}'",
        dictionary_id.green()
    ));

    Ok(())
}

async fn add_rules(
    client: &Client,
    api_key: &str,
    dictionary_id: &str,
    rules_file: &str,
) -> Result<()> {
    let file_path = Path::new(rules_file);

    if !file_path.exists() {
        return Err(anyhow::anyhow!("Rules file not found: {}", rules_file));
    }

    let rules_content = std::fs::read_to_string(file_path)
        .context("Failed to read rules file")?;

    // Parse to validate JSON
    let _rules: serde_json::Value = serde_json::from_str(&rules_content)
        .context("Failed to parse rules JSON")?;

    print_info(&format!(
        "Adding rules to dictionary '{}'...",
        dictionary_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation/dictionaries/{}/rules",
        dictionary_id
    );
    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .body(rules_content)
        .send()
        .await
        .context("Failed to add rules")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Added rules to dictionary '{}'", dictionary_id.green()));

    Ok(())
}

async fn remove_rules(
    client: &Client,
    api_key: &str,
    dictionary_id: &str,
    rules_file: &str,
) -> Result<()> {
    let file_path = Path::new(rules_file);

    if !file_path.exists() {
        return Err(anyhow::anyhow!("Rules file not found: {}", rules_file));
    }

    let rules_content = std::fs::read_to_string(file_path)
        .context("Failed to read rules file")?;

    // Parse to validate JSON
    let _rules: serde_json::Value = serde_json::from_str(&rules_content)
        .context("Failed to parse rules JSON")?;

    print_info(&format!(
        "Removing rules from dictionary '{}'...",
        dictionary_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation/dictionaries/{}/rules",
        dictionary_id
    );
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .body(rules_content)
        .send()
        .await
        .context("Failed to remove rules")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!(
        "Removed rules from dictionary '{}'",
        dictionary_id.green()
    ));

    Ok(())
}

async fn get_pls_file(
    client: &Client,
    api_key: &str,
    dictionary_id: &str,
    output: &str,
) -> Result<()> {
    print_info(&format!(
        "Downloading PLS file for dictionary '{}'...",
        dictionary_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation/dictionaries/{}/pls",
        dictionary_id
    );
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to download PLS file")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let content = response.bytes().await.context("Failed to read PLS content")?;

    let mut file = File::create(output).context("Failed to create output file")?;
    file.write_all(&content).context("Failed to write PLS file")?;

    print_success(&format!(
        "Downloaded PLS file to '{}'",
        output.green()
    ));

    Ok(())
}
