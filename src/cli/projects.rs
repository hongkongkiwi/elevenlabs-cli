//! Projects CLI arguments

use clap::{Args, Subcommand};

/// Projects API arguments
#[derive(Args)]
pub struct ProjectsArgs {
    #[command(subcommand)]
    pub command: ProjectsCommands,
}

#[derive(Subcommand)]
pub enum ProjectsCommands {
    /// List all projects
    List {
        /// Page size
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Get project details
    Get {
        /// Project ID
        project_id: String,
    },
    /// Delete a project
    Delete {
        /// Project ID
        project_id: String,
    },
    /// Convert a project
    Convert {
        /// Project ID
        project_id: String,
    },
    /// Get project snapshots
    Snapshots {
        /// Project ID
        project_id: String,
    },
    /// Get project audio
    Audio {
        /// Project ID
        project_id: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
}
