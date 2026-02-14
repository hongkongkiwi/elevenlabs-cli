//! Usage CLI arguments

use clap::{Args, Subcommand};

/// Usage arguments
#[derive(Args)]
pub struct UsageArgs {
    #[command(subcommand)]
    pub command: UsageCommands,
}

#[derive(Subcommand)]
pub enum UsageCommands {
    /// Get usage statistics
    Stats {
        /// Start time (Unix timestamp)
        #[arg(short, long)]
        start: Option<u64>,

        /// End time (Unix timestamp)
        #[arg(short, long)]
        end: Option<u64>,

        /// Breakdown type (none, voice, user, groups, voice_multiplier)
        #[arg(short, long)]
        breakdown: Option<String>,
    },
}
