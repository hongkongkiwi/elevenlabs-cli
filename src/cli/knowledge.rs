//! Knowledge base CLI arguments

use clap::{Args, Subcommand};

/// Knowledge base arguments
#[derive(Args)]
pub struct KnowledgeArgs {
    #[command(subcommand)]
    pub command: KnowledgeCommands,
}

#[derive(Subcommand)]
pub enum KnowledgeCommands {
    /// List all documents
    List {
        /// Page size
        #[arg(short, long)]
        limit: Option<u32>,

        /// Starting offset
        #[arg(short, long)]
        offset: Option<u32>,
    },
    /// Add document from URL
    AddFromUrl {
        /// URL to fetch document from
        #[arg(short, long)]
        url: String,

        /// Document name
        #[arg(short, long)]
        name: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Add document from text
    AddFromText {
        /// Text content
        #[arg(short, long)]
        text: String,

        /// Document name
        #[arg(short, long)]
        name: String,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Add document from file
    AddFromFile {
        /// File path
        #[arg(short, long, value_name = "FILE")]
        file: String,

        /// Document name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Get document details
    Get {
        /// Document ID
        document_id: String,
    },
    /// Delete a document
    Delete {
        /// Document ID
        document_id: String,
    },
}
