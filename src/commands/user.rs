use crate::cli::{UserArgs, UserCommands};
use crate::output::{print_info, print_success};
use anyhow::Result;
use colored::*;
use elevenlabs_rs::{
    endpoints::admin::user::{GetUserInfo, GetUserSubscriptionInfo},
    ElevenLabsClient,
};

pub async fn execute(args: UserArgs, api_key: &str) -> Result<()> {
    let client = ElevenLabsClient::new(api_key);

    match args.command {
        UserCommands::Info => get_user_info(&client).await?,
        UserCommands::Subscription => get_subscription(&client).await?,
    }

    Ok(())
}

async fn get_user_info(client: &ElevenLabsClient) -> Result<()> {
    print_info("Fetching user information...");

    let endpoint = GetUserInfo;
    let user = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("\n{}", "User Information:".bold().underline());
    println!("  User ID: {}", user.user_id.cyan());
    println!("  Subscription: {}", user.subscription.tier.yellow());
    println!(
        "  Character count: {}",
        user.subscription.character_count.to_string().green()
    );
    println!(
        "  Character limit: {}",
        user.subscription.character_limit.to_string().green()
    );

    if user.subscription.character_limit > 0 {
        let percentage = (user.subscription.character_count as f64
            / user.subscription.character_limit as f64)
            * 100.0;
        println!("  Usage: {:.1}%", percentage);
    }

    if user.subscription.next_character_count_reset_unix > 0 {
        println!(
            "  Next reset: {}s",
            user.subscription
                .next_character_count_reset_unix
                .to_string()
                .dimmed()
        );
    }

    print_success("User information retrieved");
    Ok(())
}

async fn get_subscription(client: &ElevenLabsClient) -> Result<()> {
    print_info("Fetching subscription details...");

    let endpoint = GetUserSubscriptionInfo;
    let subscription = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    println!("\n{}", "Subscription Details:".bold().underline());
    println!("  Tier: {}", subscription.tier.yellow());
    println!(
        "  Character count: {}",
        subscription.character_count.to_string().green()
    );
    println!(
        "  Character limit: {}",
        subscription.character_limit.to_string().green()
    );

    if subscription.character_limit > 0 {
        let percentage =
            (subscription.character_count as f64 / subscription.character_limit as f64) * 100.0;
        println!("  Usage: {:.1}%", percentage);
    }

    if subscription.voice_limit > 0 {
        println!("  Voice limit: {}", subscription.voice_limit);
    }

    if subscription.professional_voice_limit > 0 {
        println!(
            "  Professional voices limit: {}",
            subscription.professional_voice_limit
        );
    }

    if subscription.next_character_count_reset_unix > 0 {
        println!(
            "  Next character reset: {}s",
            subscription
                .next_character_count_reset_unix
                .to_string()
                .dimmed()
        );
    }

    print_success("Subscription details retrieved");
    Ok(())
}
