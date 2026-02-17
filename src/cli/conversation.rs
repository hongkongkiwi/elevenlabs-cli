//! Conversation CLI arguments

use clap::{Args, Subcommand};

/// Conversation arguments
#[derive(Args)]
pub struct ConversationArgs {
    #[command(subcommand)]
    pub command: ConversationCommands,
}

#[derive(Args)]
pub struct ConverseArgs {
    /// Agent ID to converse with
    #[arg(long)]
    pub agent_id: String,

    /// Initial user message
    #[arg(short, long)]
    pub message: Option<String>,

    /// Audio input file (for testing)
    #[arg(long, value_name = "FILE")]
    pub audio: Option<String>,

    /// Maximum conversation turns
    #[arg(long)]
    pub max_turns: Option<u32>,

    /// Play agent audio responses in real-time
    #[arg(long)]
    pub play: bool,

    /// Output audio device name
    #[arg(long, value_name = "DEVICE")]
    pub output_device: Option<String>,
}

#[derive(Subcommand)]
pub enum ConversationCommands {
    /// Start a WebSocket conversation with an agent
    #[command(name = "chat")]
    Converse(ConverseArgs),

    /// List conversations
    List {
        /// Filter by agent ID
        #[arg(long)]
        agent_id: Option<String>,

        /// Filter by branch ID
        #[arg(long)]
        branch_id: Option<String>,

        /// Page size
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// Get conversation details
    Get {
        /// Conversation ID
        conversation_id: String,
    },

    /// Get signed URL for conversation
    SignedUrl {
        /// Agent ID
        agent_id: String,

        /// Branch ID filter
        #[arg(short, long)]
        branch_id: Option<String>,
    },

    /// Get conversation token
    Token {
        /// Agent ID
        agent_id: String,

        /// Branch ID filter
        #[arg(short, long)]
        branch_id: Option<String>,
    },

    /// Delete a conversation
    Delete {
        /// Conversation ID
        conversation_id: String,
    },

    /// Download conversation audio
    Audio {
        /// Conversation ID
        conversation_id: String,

        /// Output file path
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,
    },

    /// Send feedback on a conversation
    Feedback {
        /// Conversation ID
        conversation_id: String,

        /// Thumbs up or down
        #[arg(short, long)]
        thumbs_up: bool,

        /// Optional feedback text
        #[arg(short, long)]
        feedback: Option<String>,
    },

    /// Make an outbound call via Twilio
    Outbound {
        /// Agent ID
        #[arg(long)]
        agent_id: String,

        /// Caller ID (your Twilio phone number)
        #[arg(long)]
        caller_id: String,

        /// Phone number to call
        #[arg(long)]
        to: String,

        /// Initial greeting/message
        #[arg(short, long)]
        message: Option<String>,
    },
}
