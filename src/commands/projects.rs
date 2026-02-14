//! Projects API commands for Audio Native projects.
//!
//! This module implements the Projects API for managing audio projects.
//! API Reference: https://elevenlabs.io/docs/api-reference/projects

use crate::cli::{ProjectsArgs, ProjectsCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use crate::utils::{confirm_overwrite, write_bytes_to_file};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;

pub async fn execute(args: ProjectsArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    let client = create_http_client();

    match args.command {
        ProjectsCommands::List { limit } => list_projects(&client, api_key, limit).await,
        ProjectsCommands::Get { project_id } => get_project(&client, api_key, &project_id).await,
        ProjectsCommands::Delete { project_id } => {
            delete_project(&client, api_key, &project_id).await
        }
        ProjectsCommands::Convert { project_id } => {
            convert_project(&client, api_key, &project_id).await
        }
        ProjectsCommands::Snapshots { project_id } => {
            list_snapshots(&client, api_key, &project_id).await
        }
        ProjectsCommands::Audio { project_id, output } => {
            download_project_audio(&client, api_key, &project_id, output, assume_yes).await
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProjectInfo {
    project_id: String,
    name: String,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    voice_id: Option<String>,
    #[serde(default)]
    model_id: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ProjectsListResponse {
    projects: Vec<ProjectInfo>,
}

#[derive(Debug, Deserialize)]
struct SnapshotInfo {
    snapshot_id: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SnapshotsResponse {
    snapshots: Vec<SnapshotInfo>,
}

async fn list_projects(client: &Client, api_key: &str, limit: Option<u32>) -> Result<()> {
    print_info("Fetching projects...");

    let mut url = "https://api.elevenlabs.io/v1/projects".to_string();

    if let Some(l) = limit {
        url.push_str(&format!("?page_size={}", l));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch projects")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let projects_response: ProjectsListResponse =
        response.json().await.context("Failed to parse response")?;

    if projects_response.projects.is_empty() {
        print_info("No projects found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["ID", "Name", "State", "Duration"]);

    for project in &projects_response.projects {
        let state = project.state.as_deref().unwrap_or("unknown");
        let duration = project
            .duration
            .map(|d| format!("{:.1}s", d))
            .unwrap_or_default();
        table.add_row(vec![
            project.project_id.yellow(),
            project.name.cyan(),
            state.into(),
            duration.into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!(
        "Found {} projects",
        projects_response.projects.len()
    ));

    Ok(())
}

async fn get_project(client: &Client, api_key: &str, project_id: &str) -> Result<()> {
    print_info(&format!("Fetching project '{}'...", project_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/projects/{}", project_id);
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let project: ProjectInfo = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["ID", &project.project_id.yellow()]);
    table.add_row(vec!["Name", &project.name.cyan()]);

    if let Some(ref state) = project.state {
        table.add_row(vec!["State", state]);
    }
    if let Some(ref created) = project.created_at {
        table.add_row(vec!["Created", created]);
    }
    if let Some(ref voice) = project.voice_id {
        table.add_row(vec!["Voice ID", voice]);
    }
    if let Some(ref model) = project.model_id {
        table.add_row(vec!["Model ID", model]);
    }
    if let Some(duration) = project.duration {
        table.add_row(vec!["Duration", &format!("{:.1}s", duration)]);
    }

    println!("{}", table);
    Ok(())
}

async fn delete_project(client: &Client, api_key: &str, project_id: &str) -> Result<()> {
    print_info(&format!("Deleting project '{}'...", project_id.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/projects/{}", project_id);
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
        "Project '{}' deleted successfully",
        project_id.green()
    ));
    Ok(())
}

async fn convert_project(client: &Client, api_key: &str, project_id: &str) -> Result<()> {
    print_info(&format!("Converting project '{}'...", project_id.cyan()));

    let url = format!(
        "https://api.elevenlabs.io/v1/projects/{}/convert",
        project_id
    );
    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!(
        "Project '{}' conversion started",
        project_id.green()
    ));
    Ok(())
}

async fn list_snapshots(client: &Client, api_key: &str, project_id: &str) -> Result<()> {
    print_info(&format!(
        "Fetching snapshots for project '{}'...",
        project_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/projects/{}/snapshots",
        project_id
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

    let snapshots_response: SnapshotsResponse = response.json().await?;

    if snapshots_response.snapshots.is_empty() {
        print_info("No snapshots found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Snapshot ID", "Status", "Created", "Duration"]);

    for snapshot in &snapshots_response.snapshots {
        let status = snapshot.status.as_deref().unwrap_or("unknown");
        let created = snapshot.created_at.as_deref().unwrap_or("-");
        let duration = snapshot
            .duration
            .map(|d| format!("{:.1}s", d))
            .unwrap_or_default();

        table.add_row(vec![
            snapshot.snapshot_id.yellow(),
            status.into(),
            created.into(),
            duration.into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!(
        "Found {} snapshots",
        snapshots_response.snapshots.len()
    ));

    Ok(())
}

async fn download_project_audio(
    client: &Client,
    api_key: &str,
    project_id: &str,
    output: Option<String>,
    assume_yes: bool,
) -> Result<()> {
    print_info(&format!(
        "Downloading audio for project '{}'...",
        project_id.cyan()
    ));

    let url = format!("https://api.elevenlabs.io/v1/projects/{}/audio", project_id);
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let audio = response.bytes().await?;

    let output_path = output.unwrap_or_else(|| format!("{}.mp3", project_id));

    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    write_bytes_to_file(&audio, path)?;

    print_success(&format!("Audio downloaded -> {}", output_path.green()));

    Ok(())
}
