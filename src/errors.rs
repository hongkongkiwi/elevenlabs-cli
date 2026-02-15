//! Error handling utilities for ElevenLabs CLI

use colored::*;
use std::fmt;

/// API Error types for better error messages
#[derive(Debug)]
pub enum ApiError {
    /// Missing or empty API key
    MissingApiKey,
    /// Invalid API key (401)
    Unauthorized,
    /// Permission denied (403)
    Forbidden(String),
    /// Resource not found (404)
    NotFound(String),
    /// Rate limited (429)
    RateLimited,
    /// Server error (5xx)
    ServerError(u16, String),
    /// Network error
    NetworkError(String),
    /// Other errors
    Other(String),
}

impl std::error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::MissingApiKey => write!(f, "API key is required"),
            ApiError::Unauthorized => write!(f, "Invalid API key"),
            ApiError::Forbidden(feature) => write!(f, "Permission denied: {}", feature),
            ApiError::NotFound(resource) => write!(f, "Not found: {}", resource),
            ApiError::RateLimited => write!(f, "Rate limited"),
            ApiError::ServerError(code, msg) => write!(f, "Server error {}: {}", code, msg),
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

/// Check if the error is due to missing API key
pub fn is_missing_api_key(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("api key") && (error_str.contains("required") || error_str.contains("none"))
}

/// Check if the error is due to unauthorized/invalid API key (401)
pub fn is_unauthorized(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("401")
        || error_str.contains("unauthorized")
        || error_str.contains("invalid api key")
}

/// Check if the error is due to permission denied (403)
pub fn is_forbidden(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("403")
        || error_str.contains("forbidden")
        || error_str.contains("permission denied")
}

/// Check if the error is due to not found (404)
pub fn is_not_found(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("404") || error_str.contains("not found")
}

/// Check if the error is due to rate limiting (429)
pub fn is_rate_limited(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("429")
        || error_str.contains("rate limit")
        || error_str.contains("too many requests")
}

/// Print error with helpful guidance based on error type
pub fn print_api_error(err: &anyhow::Error) {
    let error_str = err.to_string();

    if is_missing_api_key(err) {
        eprintln!("{}", "Error: API key is required".red());
        eprintln!();
        eprintln!("{}", "To get an API key:".yellow());
        eprintln!("  1. Go to https://elevenlabs.io/app/settings/api-keys");
        eprintln!("  2. Create a new API key");
        eprintln!("  3. Copy it and set it as ELEVENLABS_API_KEY environment variable");
        eprintln!();
        eprintln!("{}", "Or use the --api-key flag:".yellow());
        eprintln!("  elevenlabs --api-key YOUR_API_KEY ...");
    } else if is_unauthorized(err) {
        eprintln!("{}", "Error: Invalid API key".red());
        eprintln!();
        eprintln!("{}", "Your API key may be invalid or expired.".yellow());
        eprintln!("Get a new key from: https://elevenlabs.io/app/settings/api-keys");
    } else if is_forbidden(err) {
        eprintln!("{}", "Error: Permission denied".red());
        eprintln!();
        eprintln!(
            "{}",
            "Your API key doesn't have permission for this feature.".yellow()
        );
        eprintln!("Check your subscription tier at: https://elevenlabs.io/app/settings");
        eprintln!();
        eprintln!("Some features require:");
        eprintln!("  - Paid subscription (for professional voice cloning)");
        eprintln!("  - Business subscription (for agents, phone, etc.)");
        eprintln!("See: https://elevenlabs.io/pricing");
    } else if is_not_found(err) {
        eprintln!("{}", "Error: Resource not found".red());
        eprintln!("The requested resource doesn't exist or has been deleted.");
    } else if is_rate_limited(err) {
        eprintln!("{}", "Error: Rate limited".red());
        eprintln!("Too many requests. Please wait a moment and try again.");
    } else if error_str.contains("connection") || error_str.contains("network") {
        eprintln!("{}", "Error: Network error".red());
        eprintln!("Could not connect to ElevenLabs API.");
        eprintln!("Check your internet connection and try again.");
    } else {
        eprintln!("{}", format!("Error: {}", err).red());
    }
}

/// Helper to check API key and print helpful message if missing
pub fn check_api_key(api_key: &Option<String>) -> Option<&String> {
    match api_key {
        Some(key) if !key.is_empty() => Some(key),
        _ => {
            eprintln!("{}", "Error: API key is required".red());
            eprintln!();
            eprintln!("{}", "Set your API key using:".yellow());
            eprintln!("  export ELEVENLABS_API_KEY=your_api_key");
            eprintln!();
            eprintln!("Or use the --api-key flag:");
            eprintln!("  elevenlabs --api-key YOUR_API_KEY <command>");
            eprintln!();
            eprintln!("Get your API key from: https://elevenlabs.io/app/settings/api-keys");
            None
        }
    }
}

/// Print feature availability based on subscription tier
pub fn print_subscription_info(tier: &str) {
    println!("\n{}", "Your Subscription:".bold().underline());
    println!("  Tier: {}", tier.yellow());
    println!();

    println!("{}", "Feature Availability:".bold());

    let features = match tier.to_lowercase().as_str() {
        "free" => vec![
            ("TTS (Basic)", "✓", Some("Limited voices")),
            ("STT", "✓", Some("Limited usage")),
            ("Voice Library", "✓", Some("Browse only")),
            (
                "Professional Voice Cloning",
                "✗",
                Some("Upgrade to Starter"),
            ),
            ("Agents", "✗", Some("Upgrade to Starter")),
            ("Phone Numbers", "✗", Some("Upgrade to Starter")),
            ("Custom Pronunciations", "✗", Some("Upgrade to Starter")),
        ],
        "starter" => vec![
            ("TTS (All Voices)", "✓", None),
            ("STT", "✓", None),
            ("Voice Library", "✓", None),
            ("Professional Voice Cloning", "✓", None),
            ("Voice Fine-tuning", "✓", None),
            ("Agents", "✗", Some("Upgrade to Creator")),
            ("Phone Numbers", "✗", Some("Upgrade to Creator")),
            ("Custom Pronunciations", "✗", Some("Upgrade to Creator")),
        ],
        "creator" | "pro" | "business" => vec![
            ("All TTS Features", "✓", None),
            ("All STT Features", "✓", None),
            ("Voice Library", "✓", None),
            ("Professional Voice Cloning", "✓", None),
            ("Voice Fine-tuning", "✓", None),
            ("Agents", "✓", None),
            ("Phone Numbers", "✓", None),
            ("Custom Pronunciations", "✓", None),
            ("Priority Support", "✓", None),
        ],
        _ => vec![
            ("TTS", "?", Some("Check your dashboard")),
            ("STT", "?", Some("Check your dashboard")),
            ("Voice Library", "?", Some("Check your dashboard")),
            ("Agents", "?", Some("Check your dashboard")),
        ],
    };

    for (feature, status, note) in features {
        let status_colored = if status == "✓" {
            status.green()
        } else if status == "✗" {
            status.red()
        } else {
            status.yellow()
        };

        if let Some(n) = note {
            println!("  {} {} - {}", status_colored, feature, n.dimmed());
        } else {
            println!("  {} {}", status_colored, feature);
        }
    }

    println!();
    println!("{}", "View full feature comparison:".dimmed());
    println!("  https://elevenlabs.io/pricing");
}
