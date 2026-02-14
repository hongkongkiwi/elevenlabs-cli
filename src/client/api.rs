//! Unified API client for ElevenLabs

use crate::utils::DEFAULT_TIMEOUT_SECS;
use reqwest::Client;
use std::time::Duration;

/// Create an HTTP client with proper timeout configuration
pub fn create_http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| Client::new())
}
