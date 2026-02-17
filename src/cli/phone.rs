//! Phone Numbers CLI arguments

use clap::{Args, Subcommand};

/// Phone number management arguments
#[derive(Args)]
pub struct PhoneArgs {
    #[command(subcommand)]
    pub command: PhoneCommands,
}

#[derive(Subcommand)]
pub enum PhoneCommands {
    /// List all phone numbers
    List {
        /// Filter by agent ID
        #[arg(long)]
        agent_id: Option<String>,
    },
    /// Get phone number details
    Get {
        /// Phone number ID
        phone_id: String,
    },
    /// Import a phone number (Twilio or SIP)
    Import {
        /// Phone number in E.164 format (e.g., +1234567890)
        number: String,
        /// Provider type (twilio or sip)
        #[arg(short, long, value_enum)]
        provider: ProviderType,
        /// Label for the phone number
        #[arg(short, long)]
        label: Option<String>,
        /// Twilio Account SID (required for twilio provider)
        #[arg(long)]
        sid: Option<String>,
        /// Twilio Auth Token (required for twilio provider)
        #[arg(long)]
        token: Option<String>,
        /// SIP trunk URI (required for sip provider)
        #[arg(long)]
        sip_uri: Option<String>,
    },
    /// Update phone number settings
    Update {
        /// Phone number ID
        phone_id: String,
        /// New label
        #[arg(short, long)]
        label: Option<String>,
        /// Agent ID to link
        #[arg(long)]
        agent_id: Option<String>,
    },
    /// Delete a phone number
    Delete {
        /// Phone number ID
        phone_id: String,
    },
    /// Make a test call to a phone number
    Test {
        /// Phone number ID
        phone_id: String,
        /// Agent ID to use for the test call (optional, uses default agent if not provided)
        #[arg(long)]
        agent_id: Option<String>,
    },
}

/// Provider type for phone number import
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ProviderType {
    /// Twilio provider
    Twilio,
    /// SIP trunk provider
    Sip,
}
