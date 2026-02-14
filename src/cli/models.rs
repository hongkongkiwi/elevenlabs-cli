//! Models CLI arguments

use clap::{Args, Subcommand};

/// Model arguments
#[derive(Args)]
pub struct ModelsArgs {
    #[command(subcommand)]
    pub command: ModelsCommands,
}

#[derive(Subcommand)]
pub enum ModelsCommands {
    /// List all available models
    List,
    /// Get model pricing/rates
    Rates,
}
