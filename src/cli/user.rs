//! User CLI arguments

use clap::{Args, Subcommand};

/// User arguments
#[derive(Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCommands,
}

#[derive(Subcommand)]
pub enum UserCommands {
    /// Get user information
    Info,
    /// Get user subscription details
    Subscription,
}
