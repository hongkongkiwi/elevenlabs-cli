//! Webhook CLI arguments

use clap::{Args, Subcommand};

/// Webhook arguments
#[derive(Args)]
pub struct WebhookArgs {
    #[command(subcommand)]
    pub command: WebhookCommands,
}

#[derive(Subcommand)]
pub enum WebhookCommands {
    /// List all webhooks
    List,
    /// Create a new webhook
    Create {
        /// Webhook name
        #[arg(short, long)]
        name: String,

        /// Webhook URL
        #[arg(short, long)]
        url: String,

        /// Events to subscribe to
        #[arg(short, long, value_delimiter = ',')]
        events: Vec<String>,
    },
    /// Delete a webhook
    Delete {
        /// Webhook ID
        webhook_id: String,
    },
}
