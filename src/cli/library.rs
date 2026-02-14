//! Voice Library CLI arguments

use clap::{Args, Subcommand};

/// Voice Library arguments
#[derive(Args)]
pub struct VoiceLibraryArgs {
    #[command(subcommand)]
    pub command: VoiceLibraryCommands,
}

#[derive(Subcommand)]
pub enum VoiceLibraryCommands {
    /// List shared voices in the library
    List {
        /// Page size
        #[arg(short, long)]
        page_size: Option<u32>,

        /// Category filter (professional, high_quality, cloned, premade, generated)
        #[arg(short, long)]
        category: Option<String>,

        /// Gender filter (male, female)
        #[arg(short, long)]
        gender: Option<String>,

        /// Age filter (young, middle_aged, old)
        #[arg(long)]
        age: Option<String>,

        /// Language filter
        #[arg(short, long)]
        language: Option<String>,

        /// Accent filter
        #[arg(long)]
        accent: Option<String>,

        /// Use cases filter
        #[arg(long)]
        use_cases: Option<String>,

        /// Descriptives filter
        #[arg(long)]
        descriptives: Option<String>,

        /// Search query
        #[arg(short, long)]
        search: Option<String>,

        /// Show only featured voices
        #[arg(long)]
        featured: bool,
    },
    /// List saved voices (voices added to collections)
    Saved {
        /// Page size
        #[arg(short, long)]
        page_size: Option<u32>,
    },
    /// Add a shared voice to your library
    Add {
        /// Public user ID of the voice owner
        #[arg(short, long)]
        public_user_id: String,

        /// Voice ID
        #[arg(short, long)]
        voice_id: String,

        /// Name for the voice in your library
        #[arg(short, long)]
        name: String,
    },
    /// List voice collections
    Collections {
        /// Page size
        #[arg(short, long)]
        page_size: Option<u32>,
    },
    /// Get voices in a collection
    CollectionVoices {
        /// Collection ID
        collection_id: String,
    },
}
