//! Text output utilities with colored formatting

use colored::*;
use std::sync::atomic::{AtomicBool, Ordering};

static JSON_MODE: AtomicBool = AtomicBool::new(false);

/// Set JSON output mode globally
pub fn set_json_mode(enabled: bool) {
    JSON_MODE.store(enabled, Ordering::SeqCst);
}

/// Check if JSON mode is enabled
pub fn is_json_mode() -> bool {
    JSON_MODE.load(Ordering::SeqCst)
}

/// Print success message with green checkmark (or JSON output)
pub fn print_success(message: &str) {
    if is_json_mode() {
        println!(
            "{}",
            serde_json::json!({"status": "success", "message": message})
        );
    } else {
        println!("{} {}", "✓".green(), message);
    }
}

/// Print error message with red X (or JSON output)
pub fn print_error(message: &str) {
    if is_json_mode() {
        eprintln!(
            "{}",
            serde_json::json!({"status": "error", "message": message})
        );
    } else {
        eprintln!("{} {}", "✗".red(), message);
    }
}

/// Print info message with blue info symbol (or JSON output)
pub fn print_info(message: &str) {
    if is_json_mode() {
        println!(
            "{}",
            serde_json::json!({"status": "info", "message": message})
        );
    } else {
        println!("{} {}", "ℹ".blue(), message);
    }
}

/// Print warning message with yellow warning symbol (or JSON output)
pub fn print_warning(message: &str) {
    if is_json_mode() {
        println!(
            "{}",
            serde_json::json!({"status": "warning", "message": message})
        );
    } else {
        println!("{} {}", "⚠".yellow(), message);
    }
}
