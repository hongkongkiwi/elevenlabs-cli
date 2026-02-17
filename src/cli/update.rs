//! Update CLI arguments

use clap::Args;

/// Update arguments
#[derive(Args, Clone)]
pub struct UpdateArgs {
    /// Force update even if already up to date
    #[arg(long)]
    pub force: bool,

    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,
}
