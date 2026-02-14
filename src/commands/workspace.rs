use crate::cli::{WorkspaceArgs, WorkspaceCommands};
use crate::client::create_http_client;
use crate::output::{print_error, print_info, print_success, print_warning};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

pub async fn execute(args: WorkspaceArgs, api_key: &str) -> Result<()> {
    let client = create_http_client();

    match args.command {
        WorkspaceCommands::Info => get_workspace_info(&client, api_key).await,
        WorkspaceCommands::Members => list_members(&client, api_key).await,
        WorkspaceCommands::RemoveMember { user_id } => {
            remove_member(&client, api_key, &user_id).await
        }
        WorkspaceCommands::Invites => list_invites(&client, api_key).await,
        WorkspaceCommands::Invite { email, role } => {
            invite_member(&client, api_key, &email, &role).await
        }
        WorkspaceCommands::Revoke { email } => revoke_invite(&client, api_key, &email).await,
        WorkspaceCommands::ApiKeys => list_api_keys(&client, api_key).await,
        WorkspaceCommands::Secrets => list_secrets(&client, api_key).await,
        WorkspaceCommands::AddSecret {
            name,
            value,
            secret_type,
        } => add_secret(&client, api_key, &name, &value, &secret_type).await,
        WorkspaceCommands::DeleteSecret { name } => delete_secret(&client, api_key, &name).await,
        WorkspaceCommands::Share {
            resource_type,
            resource_id,
            share_option,
        } => {
            share_resource(
                &client,
                api_key,
                &resource_type,
                &resource_id,
                &share_option,
            )
            .await
        }
        WorkspaceCommands::Unshare {
            resource_type,
            resource_id,
        } => unshare_resource(&client, api_key, &resource_type, &resource_id).await,
    }
}

#[derive(Debug, Deserialize)]
struct WorkspaceInfo {
    id: String,
    name: String,
    #[serde(default)]
    owner_id: Option<String>,
    #[serde(default)]
    member_count: Option<u32>,
    #[serde(default)]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InviteInfo {
    email: String,
    role: String,
    #[serde(default)]
    invited_at: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    invitation_id: Option<String>,
}

async fn get_workspace_info(client: &Client, api_key: &str) -> Result<()> {
    print_info("Fetching workspace information...");

    let response = client
        .get("https://api.elevenlabs.io/v1/workspace")
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch workspace info")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        print_error(&format!("API error: {}", error));
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let workspace: WorkspaceInfo = response.json().await.context("Failed to parse response")?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["ID", &workspace.id.yellow()]);
    table.add_row(vec!["Name", &workspace.name.cyan()]);

    if let Some(owner_id) = &workspace.owner_id {
        table.add_row(vec!["Owner ID", owner_id]);
    }
    if let Some(count) = workspace.member_count {
        table.add_row(vec!["Members", &count.to_string()]);
    }
    if let Some(created) = &workspace.created_at {
        table.add_row(vec!["Created", created]);
    }

    println!("{}", table);
    Ok(())
}

async fn list_invites(client: &Client, api_key: &str) -> Result<()> {
    print_info("Fetching workspace invites...");

    let response = client
        .get("https://api.elevenlabs.io/v1/workspace/invites")
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch invites")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        print_error(&format!("API error: {}", error));
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let invites: Vec<InviteInfo> = response.json().await.context("Failed to parse response")?;

    if invites.is_empty() {
        print_success("No pending invites");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Email", "Role", "Invited At"]);

    for invite in &invites {
        table.add_row(vec![
            invite.email.cyan(),
            invite.role.yellow(),
            invite.invited_at.as_deref().unwrap_or("-").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} pending invite(s)", invites.len()));
    Ok(())
}

async fn invite_member(client: &Client, api_key: &str, email: &str, role: &str) -> Result<()> {
    // Validate role
    let valid_roles = ["admin", "editor", "viewer", "contributor"];
    if !valid_roles.contains(&role.to_lowercase().as_str()) {
        return Err(anyhow::anyhow!(
            "Invalid role '{}'. Valid roles are: {}",
            role,
            valid_roles.join(", ")
        ));
    }

    print_info(&format!(
        "Inviting '{}' as '{}'...",
        email.cyan(),
        role.yellow()
    ));

    let body = json!({
        "email": email,
        "role": role.to_lowercase()
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/workspace/invites")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to send invite")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Invitation sent to '{}'", email.green()));
    Ok(())
}

async fn revoke_invite(client: &Client, api_key: &str, email: &str) -> Result<()> {
    print_info(&format!("Revoking invitation for '{}'...", email.cyan()));

    let url = format!("https://api.elevenlabs.io/v1/workspace/invites/{}", email);
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to revoke invite")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Invitation revoked for '{}'", email.green()));
    Ok(())
}

async fn list_members(client: &Client, api_key: &str) -> Result<()> {
    print_info("Fetching workspace members...");

    let response = client
        .get("https://api.elevenlabs.io/v1/workspace/members")
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch members")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        print_error(&format!("API error: {}", error));
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct MemberInfo {
        user_id: String,
        email: String,
        role: String,
        #[serde(default)]
        joined_at: Option<String>,
    }

    let members: Vec<MemberInfo> = response.json().await.context("Failed to parse response")?;

    if members.is_empty() {
        print_info("No members found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["User ID", "Email", "Role", "Joined"]);

    for member in &members {
        table.add_row(vec![
            member.user_id.yellow(),
            member.email.cyan(),
            member.role.green(),
            member.joined_at.as_deref().unwrap_or("-").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} member(s)", members.len()));
    Ok(())
}

async fn remove_member(client: &Client, api_key: &str, user_id: &str) -> Result<()> {
    print_warning(&format!(
        "You are about to remove member '{}' from the workspace",
        user_id
    ));

    let confirm = dialoguer::Confirm::new()
        .with_prompt("Are you sure?")
        .default(false)
        .interact()?;

    if !confirm {
        print_info("Cancelled");
        return Ok(());
    }

    let url = format!("https://api.elevenlabs.io/v1/workspace/members/{}", user_id);
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to remove member")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!(
        "Member '{}' removed successfully",
        user_id.green()
    ));
    Ok(())
}

async fn list_api_keys(client: &Client, api_key: &str) -> Result<()> {
    print_info("Fetching workspace API keys...");

    let response = client
        .get("https://api.elevenlabs.io/v1/workspace/api-keys")
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch API keys")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        print_error(&format!("API error: {}", error));
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct ApiKeyInfo {
        key_id: String,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        created_at: Option<String>,
        #[serde(default)]
        last_used_at: Option<String>,
    }

    let keys: Vec<ApiKeyInfo> = response.json().await.context("Failed to parse response")?;

    if keys.is_empty() {
        print_info("No API keys found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Key ID", "Name", "Created", "Last Used"]);

    for key in &keys {
        table.add_row(vec![
            key.key_id.yellow(),
            key.name.as_deref().unwrap_or("-").cyan(),
            key.created_at.as_deref().unwrap_or("-").into(),
            key.last_used_at.as_deref().unwrap_or("Never").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} API key(s)", keys.len()));
    Ok(())
}

async fn list_secrets(client: &Client, api_key: &str) -> Result<()> {
    print_info("Fetching workspace secrets...");

    let response = client
        .get("https://api.elevenlabs.io/v1/convai/workspaces/secrets")
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch secrets")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        print_error(&format!("API error: {}", error));
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct SecretInfo {
        name: String,
        secret_type: String,
        #[serde(default)]
        created_at: Option<String>,
    }

    let secrets: Vec<SecretInfo> = response.json().await.context("Failed to parse response")?;

    if secrets.is_empty() {
        print_info("No secrets found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Name", "Type", "Created"]);

    for secret in &secrets {
        table.add_row(vec![
            secret.name.yellow(),
            secret.secret_type.cyan(),
            secret.created_at.as_deref().unwrap_or("-").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} secret(s)", secrets.len()));
    Ok(())
}

async fn add_secret(
    client: &Client,
    api_key: &str,
    name: &str,
    value: &str,
    secret_type: &str,
) -> Result<()> {
    // Validate secret type
    let valid_types = ["api_key", "env_variable", "secret"];
    if !valid_types.contains(&secret_type.to_lowercase().as_str()) {
        return Err(anyhow::anyhow!(
            "Invalid secret type '{}'. Valid types are: {}",
            secret_type,
            valid_types.join(", ")
        ));
    }

    print_info(&format!(
        "Adding secret '{}' of type '{}'...",
        name.cyan(),
        secret_type.yellow()
    ));

    let body = json!({
        "name": name,
        "value": value,
        "secret_type": secret_type.to_lowercase()
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/convai/workspaces/secrets")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to add secret")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Secret '{}' added successfully", name.green()));
    Ok(())
}

async fn delete_secret(client: &Client, api_key: &str, name: &str) -> Result<()> {
    print_warning(&format!("You are about to delete secret '{}'", name));

    let confirm = dialoguer::Confirm::new()
        .with_prompt("Are you sure?")
        .default(false)
        .interact()?;

    if !confirm {
        print_info("Cancelled");
        return Ok(());
    }

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/workspaces/secrets/{}",
        name
    );
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to delete secret")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!("Secret '{}' deleted successfully", name.green()));
    Ok(())
}

async fn share_resource(
    client: &Client,
    api_key: &str,
    resource_type: &str,
    resource_id: &str,
    share_option: &str,
) -> Result<()> {
    // Validate share option
    let valid_options = ["viewer", "editor"];
    if !valid_options.contains(&share_option.to_lowercase().as_str()) {
        return Err(anyhow::anyhow!(
            "Invalid share option '{}'. Valid options are: {}",
            share_option,
            valid_options.join(", ")
        ));
    }

    // Validate resource type
    let valid_resource_types = [
        "agent",
        "knowledge_base",
        "flow",
        "prompt",
        "model",
        "voice",
    ];
    if !valid_resource_types.contains(&resource_type.to_lowercase().as_str()) {
        print_warning(&format!(
            "Unrecognized resource type '{}'. Valid types are: {}",
            resource_type,
            valid_resource_types.join(", ")
        ));
    }

    print_info(&format!(
        "Sharing {} '{}' with {} access...",
        resource_type.cyan(),
        resource_id.yellow(),
        share_option.green()
    ));

    let body = json!({
        "resource_type": resource_type.to_lowercase(),
        "resource_id": resource_id,
        "share_option": share_option.to_lowercase()
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/convai/workspaces/shares")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to share resource")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!(
        "Resource '{}' ({}) shared successfully",
        resource_id.green(),
        resource_type
    ));
    Ok(())
}

async fn unshare_resource(
    client: &Client,
    api_key: &str,
    resource_type: &str,
    resource_id: &str,
) -> Result<()> {
    print_warning(&format!(
        "You are about to unshare {} '{}'",
        resource_type, resource_id
    ));

    let confirm = dialoguer::Confirm::new()
        .with_prompt("Are you sure?")
        .default(false)
        .interact()?;

    if !confirm {
        print_info("Cancelled");
        return Ok(());
    }

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/workspaces/shares/{}/{}",
        resource_type, resource_id
    );
    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to unshare resource")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!(
        "Resource '{}' ({}) unshared successfully",
        resource_id.green(),
        resource_type
    ));
    Ok(())
}
