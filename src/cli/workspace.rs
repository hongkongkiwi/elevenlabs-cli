//! Workspace CLI arguments

use clap::{Args, Subcommand};

/// Workspace arguments
#[derive(Args)]
pub struct WorkspaceArgs {
    #[command(subcommand)]
    pub command: WorkspaceCommands,
}

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    /// Get workspace information
    Info,
    /// List all members
    Members,
    /// Remove a member from workspace
    RemoveMember {
        /// Member user ID
        user_id: String,
    },
    /// List pending invites
    Invites,
    /// Invite a member
    Invite {
        /// Email address
        email: String,
        /// Role (admin, editor, viewer, contributor)
        role: String,
    },
    /// Revoke an invitation
    Revoke {
        /// Email address
        email: String,
    },
    /// List API keys
    ApiKeys,
    /// List workspace secrets
    Secrets,
    /// Add a workspace secret
    AddSecret {
        /// Secret name
        name: String,
        /// Secret value
        value: String,
        /// Secret type (api_key, env_variable, secret)
        secret_type: String,
    },
    /// Delete a workspace secret
    DeleteSecret {
        /// Secret name
        name: String,
    },
    /// Share a workspace resource
    Share {
        /// Resource type (agent, knowledge_base, etc.)
        resource_type: String,
        /// Resource ID to share
        resource_id: String,
        /// Share option (viewer, editor)
        share_option: String,
    },
    /// Unshare a workspace resource
    Unshare {
        /// Resource type (agent, knowledge_base, etc.)
        resource_type: String,
        /// Resource ID to unshare
        resource_id: String,
    },
}
