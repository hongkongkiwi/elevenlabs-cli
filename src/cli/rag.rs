//! RAG CLI arguments

use clap::{Args, Subcommand};

/// RAG Index arguments
#[derive(Args)]
pub struct RagArgs {
    #[command(subcommand)]
    pub command: RagCommands,
}

#[derive(Subcommand)]
pub enum RagCommands {
    /// Create RAG index for a document
    Create {
        /// Document ID
        #[arg(short, long)]
        document_id: String,

        /// Embedding model
        #[arg(short, long, default_value = "e5_mistil_7b_instruct")]
        model: String,
    },
    /// Get RAG index status (requires rag_index_id)
    Status {
        /// Document ID
        #[arg(short, long)]
        document_id: String,

        /// RAG index ID
        #[arg(short, long)]
        rag_index_id: String,
    },
    /// Delete RAG index
    Delete {
        /// Document ID
        #[arg(short, long)]
        document_id: String,

        /// RAG index ID
        #[arg(short, long)]
        rag_index_id: String,
    },
    /// Rebuild RAG index for a document
    Rebuild {
        /// Document ID
        #[arg(short, long)]
        document_id: String,
    },
    /// Check RAG index status for a document
    IndexStatus {
        /// Document ID
        #[arg(short, long)]
        document_id: String,
    },
}
