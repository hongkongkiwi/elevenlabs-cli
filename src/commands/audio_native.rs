use crate::cli::{AudioNativeArgs, AudioNativeCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::path::Path;

pub async fn execute(args: AudioNativeArgs, api_key: &str) -> Result<()> {
    let client = create_http_client();

    match args.command {
        AudioNativeCommands::List { limit, page } => {
            list_audio_native(&client, api_key, limit, page).await
        }
        AudioNativeCommands::Get { project_id } => {
            get_audio_native(&client, api_key, &project_id).await
        }
        AudioNativeCommands::Create {
            name,
            author,
            title,
            image,
            voice_id,
            model_id,
            file,
            small,
            text_color,
            background_color,
            auto_convert,
        } => {
            create_audio_native(
                &client,
                api_key,
                &name,
                author.as_deref(),
                title.as_deref(),
                image.as_deref(),
                voice_id.as_deref(),
                model_id.as_deref(),
                file.as_deref(),
                small,
                text_color.as_deref(),
                background_color.as_deref(),
                auto_convert,
            )
            .await
        }
    }
}

async fn list_audio_native(
    client: &Client,
    api_key: &str,
    limit: u32,
    page: u32,
) -> Result<()> {
    print_info("Fetching audio native projects...");

    let url = "https://api.elevenlabs.io/v1/audio-native";
    let request = client
        .get(url)
        .header("xi-api-key", api_key)
        .query(&[("limit", limit.to_string())])
        .query(&[("page", page.to_string())]);

    let response = request.send().await.context("Failed to fetch audio native projects")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: AudioNativeListResponse =
        response.json().await.context("Failed to parse response")?;

    let projects = result.projects;

    if projects.is_empty() {
        print_info("No audio native projects found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "Author", "Created At"]);

    for project in &projects {
        let author = project.author.clone().unwrap_or_default();
        let name = project.name.clone();
        let created_at = project.created_at.clone();
        let project_id = format!(
            "{}...",
            project.project_id.chars().take(12).collect::<String>()
        );
        table.add_row(vec![
            &project_id,
            &name.cyan().to_string(),
            &author.yellow().to_string(),
            &created_at,
        ]);
    }

    println!("\n{}", table);
    print_success(&format!("Showing {} projects", projects.len()));

    Ok(())
}

async fn get_audio_native(client: &Client, api_key: &str, project_id: &str) -> Result<()> {
    print_info(&format!(
        "Fetching audio native project '{}'...",
        project_id.cyan()
    ));

    let url = format!("https://api.elevenlabs.io/v1/audio-native/{}", project_id);
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let project: AudioNativeProject =
        response.json().await.context("Failed to parse response")?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["ID", &project.project_id.yellow()]);
    table.add_row(vec!["Name", &project.name.cyan()]);

    if let Some(author) = &project.author {
        table.add_row(vec!["Author", author]);
    }

    if let Some(title) = &project.title {
        table.add_row(vec!["Title", title]);
    }

    if let Some(voice_id) = &project.voice_id {
        table.add_row(vec!["Voice ID", voice_id]);
    }

    if let Some(model_id) = &project.model_id {
        table.add_row(vec!["Model ID", model_id]);
    }

    table.add_row(vec!["Created At", &project.created_at]);

    if let Some(html_snippet) = &project.html_snippet {
        table.add_row(vec!["Has HTML", &"Yes".green().to_string()]);
        println!("\nHTML Snippet Preview:");
        let preview: String = html_snippet.chars().take(200).collect();
        println!("  {}...", preview);
    }

    println!("{}", table);
    Ok(())
}

async fn create_audio_native(
    client: &Client,
    api_key: &str,
    name: &str,
    author: Option<&str>,
    title: Option<&str>,
    image: Option<&str>,
    voice_id: Option<&str>,
    model_id: Option<&str>,
    file: Option<&str>,
    small: bool,
    text_color: Option<&str>,
    background_color: Option<&str>,
    auto_convert: bool,
) -> Result<()> {
    // Check if file exists if provided
    if let Some(file_path) = file {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path));
        }
    }

    print_info(&format!("Creating Audio Native project '{}'...", name.cyan()));

    // Build body
    let mut body = json!({
        "name": name
    });

    if let Some(author) = author {
        body["author"] = json!(author);
    }

    if let Some(title) = title {
        body["title"] = json!(title);
    }

    if let Some(image) = image {
        body["image"] = json!(image);
    }

    if let Some(voice_id) = voice_id {
        body["voice_id"] = json!(voice_id);
    }

    if let Some(model_id) = model_id {
        body["model_id"] = json!(model_id);
    }

    if let Some(file) = file {
        // For file upload, we need to handle it differently
        // This is a simplified version - the actual implementation would need multipart upload
        body["file"] = json!(file);
    }

    if small {
        body["small"] = json!(true);
    }

    if let Some(text_color) = text_color {
        body["text_color"] = json!(text_color);
    }

    if let Some(background_color) = background_color {
        body["background_color"] = json!(background_color);
    }

    if auto_convert {
        body["auto_convert"] = json!(true);
    }

    let url = "https://api.elevenlabs.io/v1/audio-native";
    let response = client
        .post(url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to create audio native project")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: AudioNativeCreateResponse =
        response.json().await.context("Failed to parse response")?;

    print_success(&format!("Audio Native project created: '{}'", name));
    println!("  Project ID: {}", result.project_id.cyan());

    // Save the embeddable HTML if available
    if let Some(html_snippet) = result.html_snippet {
        let html_file = format!("{}_player.html", name.replace(' ', "_"));
        std::fs::write(&html_file, &html_snippet)?;
        println!("  HTML snippet saved to: {}", html_file.green());
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct AudioNativeListResponse {
    projects: Vec<AudioNativeProject>,
}

#[derive(Debug, Deserialize)]
struct AudioNativeProject {
    project_id: String,
    name: String,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    voice_id: Option<String>,
    #[serde(default)]
    model_id: Option<String>,
    created_at: String,
    #[serde(default)]
    html_snippet: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AudioNativeCreateResponse {
    project_id: String,
    #[serde(default)]
    html_snippet: Option<String>,
}
