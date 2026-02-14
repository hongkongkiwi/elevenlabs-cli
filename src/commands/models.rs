use crate::cli::{ModelsArgs, ModelsCommands};
use crate::output::{print_info, print_success};
use anyhow::Result;
use colored::*;
use comfy_table::Table;
use elevenlabs_rs::{endpoints::admin::models::GetModels, ElevenLabsClient};

pub async fn execute(args: ModelsArgs, api_key: &str) -> Result<()> {
    let client = ElevenLabsClient::new(api_key);
    match args.command {
        ModelsCommands::List => list_models(&client).await?,
        ModelsCommands::Rates => get_model_rates(&client).await?,
    }
    Ok(())
}

async fn list_models(client: &ElevenLabsClient) -> Result<()> {
    print_info("Fetching available models...");

    let endpoint = GetModels;
    let models = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("\n{}", "Available Models:".bold().underline());

    let mut table = Table::new();
    table.set_header(vec!["Model ID", "Name", "Description", "Languages"]);

    for model in &models {
        let languages = if model.languages.is_empty() {
            "-".to_string()
        } else {
            model
                .languages
                .iter()
                .map(|l| l.language_id.clone())
                .collect::<Vec<_>>()
                .join(", ")
                .chars()
                .take(40)
                .collect::<String>()
        };
        let description: String = model.description.chars().take(50).collect();
        table.add_row(vec![
            model.model_id.clone(),
            model.name.clone(),
            description,
            languages,
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} models", models.len()));

    Ok(())
}

pub async fn get_model_rates(client: &ElevenLabsClient) -> Result<()> {
    print_info("Fetching model pricing/rates...");

    let endpoint = GetModels;
    let models = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("\n{}", "Model Pricing/Rates:".bold().underline());

    let mut table = Table::new();
    table.set_header(vec!["Model ID", "Name", "Character Cost Multiplier"]);

    for model in &models {
        let cost_multiplier = model.model_rates.character_cost_multiplier;
        table.add_row(vec![
            model.model_id.clone(),
            model.name.clone(),
            format!("{:.2}x", cost_multiplier),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} models", models.len()));

    Ok(())
}
