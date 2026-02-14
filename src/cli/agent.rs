//! Agent CLI arguments

use clap::{Args, Subcommand};

/// Agent management arguments
#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommands,
}

/// Spelling patience configuration
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SpellingPatience {
    /// Automatic (default)
    Auto,
    /// Low patience
    Low,
    /// Medium patience
    Medium,
    /// High patience
    High,
}

#[derive(Subcommand)]
pub enum AgentCommands {
    /// List all agents
    List {
        /// Page size
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Get agent summaries (lightweight)
    Summaries {
        /// Page size
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Get agent details
    Get {
        /// Agent ID
        agent_id: String,
    },
    /// Create a new agent
    Create {
        /// Agent name
        #[arg(short, long)]
        name: String,

        /// Agent description
        #[arg(short, long)]
        description: Option<String>,

        /// Voice ID for the agent
        #[arg(short, long)]
        voice_id: Option<String>,

        /// First message template
        #[arg(short, long)]
        first_message: Option<String>,

        /// System prompt
        #[arg(short, long)]
        system_prompt: Option<String>,
    },
    /// Update an agent
    Update {
        /// Agent ID
        agent_id: String,

        /// New name
        #[arg(short, long)]
        name: Option<String>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete an agent
    Delete {
        /// Agent ID
        agent_id: String,
    },
    /// Get agent public link
    Link {
        /// Agent ID
        agent_id: String,
    },
    /// Duplicate an agent
    Duplicate {
        /// Agent ID to duplicate
        agent_id: String,

        /// Name for the new agent
        #[arg(short, long)]
        name: String,
    },
    /// List agent branches
    Branches {
        /// Agent ID
        agent_id: String,
    },
    /// Rename an agent branch
    RenameBranch {
        /// Agent ID
        agent_id: String,
        /// Branch ID
        branch_id: String,
        /// New branch name
        #[arg(short, long)]
        name: String,
    },
    /// List batch calls
    BatchList {
        /// Page size
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Get batch call status
    BatchStatus {
        /// Batch call ID
        batch_id: String,
    },
    /// Delete a batch call
    BatchDelete {
        /// Batch call ID
        batch_id: String,
    },
    /// Simulate a conversation with an agent (for testing)
    Simulate {
        /// Agent ID
        agent_id: String,
        /// User message to simulate
        #[arg(short, long)]
        message: String,
        /// Maximum turns in the simulation
        #[arg(long, default_value = "5")]
        max_turns: u32,
    },
    /// Update agent turn configuration
    UpdateTurn {
        /// Agent ID
        agent_id: String,
        /// Spelling patience (auto, low, medium, high)
        #[arg(long, value_enum)]
        spelling_patience: Option<SpellingPatience>,
        /// Turn silence threshold in milliseconds
        #[arg(long)]
        silence_threshold_ms: Option<u32>,
    },
    /// List WhatsApp accounts connected to agents
    WhatsappList,
}
