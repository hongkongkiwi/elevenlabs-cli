//! Config CLI arguments

use clap::{Args, Subcommand};

/// Config arguments
#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set configuration value
    Set {
        /// Configuration key (api_key, default_voice, default_model)
        key: String,
        /// Configuration value
        value: String,
    },
    /// Remove configuration value
    Unset {
        /// Configuration key
        key: String,
    },
}
