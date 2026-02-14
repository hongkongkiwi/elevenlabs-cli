use crate::cli::{UsageArgs, UsageCommands};
use crate::output::{print_info, print_success};
use anyhow::Result;
use colored::*;
use comfy_table::Table;
use elevenlabs_rs::{
    endpoints::admin::usage::{BreakdownType, GetUsage, GetUsageQuery},
    ElevenLabsClient,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn execute(args: UsageArgs, api_key: &str) -> Result<()> {
    let client = ElevenLabsClient::new(api_key);

    match args.command {
        UsageCommands::Stats {
            start,
            end,
            breakdown,
        } => get_usage_stats(&client, start, end, breakdown).await?,
    }

    Ok(())
}

async fn get_usage_stats(
    client: &ElevenLabsClient,
    start: Option<u64>,
    end: Option<u64>,
    breakdown: Option<String>,
) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Default to last 30 days if not specified
    let start_unix = start.unwrap_or(now - 30 * 24 * 60 * 60);
    let end_unix = end.unwrap_or(now);

    print_info(&format!(
        "Fetching usage stats from {} to {}...",
        start_unix.to_string().cyan(),
        end_unix.to_string().cyan()
    ));

    let mut query = GetUsageQuery::new(start_unix, end_unix);
    query = query.with_workspace_metrics(true);

    if let Some(b) = breakdown {
        let breakdown_type = match b.to_lowercase().as_str() {
            "voice" => BreakdownType::Voice,
            "user" => BreakdownType::User,
            "groups" => BreakdownType::Groups,
            "voice_multiplier" | "voice-multiplier" | "multiplier" => {
                BreakdownType::VoiceMultiplier
            }
            _ => BreakdownType::None,
        };
        query = query.with_breakdown_type(breakdown_type);
    }

    let endpoint = GetUsage::new(query);
    let response = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("\n{}", "Usage Statistics:".bold().underline());

    if response.time.is_empty() {
        print_info("No usage data found for the specified period");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Time", "Usage Type", "Characters"]);

    // The usage is a HashMap<String, Vec<u64>> where key is the usage type
    for (usage_type, values) in &response.usage {
        for (i, &value) in values.iter().enumerate() {
            if let Some(&timestamp) = response.time.get(i) {
                table.add_row(vec![
                    timestamp.to_string(),
                    usage_type.clone(),
                    value.to_string(),
                ]);
            }
        }
    }

    println!("{}", table);

    // Calculate total
    let total: u64 = response.usage.values().flatten().sum();
    print_success(&format!(
        "Total characters used: {}",
        total.to_string().green()
    ));

    Ok(())
}
