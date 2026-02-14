//! Pronunciation CLI arguments

use clap::{Args, Subcommand};

/// Pronunciation arguments
#[derive(Args)]
pub struct PronunciationArgs {
    #[command(subcommand)]
    pub command: PronunciationCommands,
}

#[derive(Subcommand)]
pub enum PronunciationCommands {
    /// List pronunciation dictionaries
    List,
    /// Add a pronunciation dictionary
    Add {
        /// PLS file path
        #[arg(short, long)]
        file: String,

        /// Dictionary name
        #[arg(short, long)]
        name: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete a pronunciation dictionary
    Delete {
        /// Dictionary ID
        dictionary_id: String,
    },
    /// List rules in a pronunciation dictionary
    Rules {
        /// Dictionary ID
        dictionary_id: String,
    },
    /// Add rules to a pronunciation dictionary
    AddRules {
        /// Dictionary ID
        dictionary_id: String,

        /// JSON file containing rules
        #[arg(short, long)]
        rules_file: String,
    },
    /// Remove rules from a pronunciation dictionary
    RemoveRules {
        /// Dictionary ID
        dictionary_id: String,

        /// JSON file containing rules
        #[arg(short, long)]
        rules_file: String,
    },
    /// Download PLS file for a pronunciation dictionary
    GetPls {
        /// Dictionary ID
        dictionary_id: String,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}
