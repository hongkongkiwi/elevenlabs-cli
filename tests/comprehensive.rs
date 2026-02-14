//! Comprehensive CLI integration tests

use std::path::PathBuf;
use std::process::Command;

/// Get binary path relative to test location
fn bin() -> PathBuf {
    // CARGO_MANIFEST_DIR points to the crate root during tests
    let manifest_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let bin_path = manifest_dir.join("target/release/elevenlabs");
    if bin_path.exists() {
        return bin_path;
    }
    let bin_path = manifest_dir.join("target/debug/elevenlabs");
    if bin_path.exists() {
        return bin_path;
    }
    manifest_dir.join("target/debug/elevenlabs")
}

/// Run CLI and check success
fn ok(args: &[&str]) -> bool {
    Command::new(bin())
        .args(args)
        .env("ELEVENLABS_API_KEY", "test")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run CLI and check failure  
fn fail(args: &[&str]) -> bool {
    !ok(args)
}

// ============================================================================
// Main Commands - 31 commands
// ============================================================================

#[test]
fn h_tts() {
    assert!(ok(&["tts", "--help"]));
}
#[test]
fn h_stt() {
    assert!(ok(&["stt", "--help"]));
}
#[test]
fn h_voice() {
    assert!(ok(&["voice", "--help"]));
}
#[test]
fn h_sfx() {
    assert!(ok(&["sfx", "--help"]));
}
#[test]
fn h_isolate() {
    assert!(ok(&["isolate", "--help"]));
}
#[test]
fn h_voice_changer() {
    assert!(ok(&["voice-changer", "--help"]));
}
#[test]
fn h_dub() {
    assert!(ok(&["dub", "--help"]));
}
#[test]
fn h_history() {
    assert!(ok(&["history", "--help"]));
}
#[test]
fn h_user() {
    assert!(ok(&["user", "--help"]));
}
#[test]
fn h_models() {
    assert!(ok(&["models", "--help"]));
}
#[test]
fn h_config() {
    assert!(ok(&["config", "--help"]));
}
#[test]
fn h_library() {
    assert!(ok(&["library", "--help"]));
}
#[test]
fn h_pronunciation() {
    assert!(ok(&["pronunciation", "--help"]));
}
#[test]
fn h_usage() {
    assert!(ok(&["usage", "--help"]));
}
#[test]
fn h_voice_design() {
    assert!(ok(&["voice-design", "--help"]));
}
#[test]
fn h_audio_native() {
    assert!(ok(&["audio-native", "--help"]));
}
#[test]
fn h_samples() {
    assert!(ok(&["samples", "--help"]));
}
#[test]
fn h_workspace() {
    assert!(ok(&["workspace", "--help"]));
}
#[test]
fn h_tts_timestamps() {
    assert!(ok(&["tts-timestamps", "--help"]));
}
#[test]
fn h_tts_stream() {
    assert!(ok(&["tts-stream", "--help"]));
}
#[test]
fn h_agent() {
    assert!(ok(&["agent", "--help"]));
}
#[test]
fn h_converse() {
    assert!(ok(&["converse", "--help"]));
}
#[test]
fn h_knowledge() {
    assert!(ok(&["knowledge", "--help"]));
}
#[test]
fn h_rag() {
    assert!(ok(&["rag", "--help"]));
}
#[test]
fn h_webhook() {
    assert!(ok(&["webhook", "--help"]));
}
#[test]
fn h_dialogue() {
    assert!(ok(&["dialogue", "--help"]));
}
#[test]
fn h_tools() {
    assert!(ok(&["tools", "--help"]));
}
#[test]
fn h_projects() {
    assert!(ok(&["projects", "--help"]));
}
#[test]
fn h_music() {
    assert!(ok(&["music", "--help"]));
}
#[test]
fn h_phone() {
    assert!(ok(&["phone", "--help"]));
}
#[test]
fn h_completions() {
    assert!(ok(&["completions", "--help"]));
}

// ============================================================================
// Voice Subcommands - 10
// ============================================================================

#[test]
fn h_voice_list() {
    assert!(ok(&["voice", "list", "--help"]));
}
#[test]
fn h_voice_get() {
    assert!(ok(&["voice", "get", "--help"]));
}
#[test]
fn h_voice_clone() {
    assert!(ok(&["voice", "clone", "--help"]));
}
#[test]
fn h_voice_settings() {
    assert!(ok(&["voice", "settings", "--help"]));
}
#[test]
fn h_voice_edit_settings() {
    assert!(ok(&["voice", "edit-settings", "--help"]));
}
#[test]
fn h_voice_fine_tune() {
    assert!(ok(&["voice", "fine-tune", "--help"]));
}
#[test]
fn h_voice_edit() {
    assert!(ok(&["voice", "edit", "--help"]));
}
#[test]
fn h_voice_share() {
    assert!(ok(&["voice", "share", "--help"]));
}
#[test]
fn h_voice_similar() {
    assert!(ok(&["voice", "similar", "--help"]));
}
#[test]
fn h_voice_delete() {
    assert!(ok(&["voice", "delete", "--help"]));
}

// ============================================================================
// History Subcommands - 5
// ============================================================================

#[test]
fn h_history_list() {
    assert!(ok(&["history", "list", "--help"]));
}
#[test]
fn h_history_get() {
    assert!(ok(&["history", "get", "--help"]));
}
#[test]
fn h_history_delete() {
    assert!(ok(&["history", "delete", "--help"]));
}
#[test]
fn h_history_download() {
    assert!(ok(&["history", "download", "--help"]));
}
#[test]
fn h_history_feedback() {
    assert!(ok(&["history", "feedback", "--help"]));
}

// ============================================================================
// Models Subcommands - 2
// ============================================================================

#[test]
fn h_models_list() {
    assert!(ok(&["models", "list", "--help"]));
}
#[test]
fn h_models_rates() {
    assert!(ok(&["models", "rates", "--help"]));
}

// ============================================================================
// Pronunciation Subcommands - 7
// ============================================================================

#[test]
fn h_pronunciation_list() {
    assert!(ok(&["pronunciation", "list", "--help"]));
}
#[test]
fn h_pronunciation_add() {
    assert!(ok(&["pronunciation", "add", "--help"]));
}
#[test]
fn h_pronunciation_delete() {
    assert!(ok(&["pronunciation", "delete", "--help"]));
}
#[test]
fn h_pronunciation_rules() {
    assert!(ok(&["pronunciation", "rules", "--help"]));
}
#[test]
fn h_pronunciation_add_rules() {
    assert!(ok(&["pronunciation", "add-rules", "--help"]));
}
#[test]
fn h_pronunciation_remove_rules() {
    assert!(ok(&["pronunciation", "remove-rules", "--help"]));
}
#[test]
fn h_pronunciation_get_pls() {
    assert!(ok(&["pronunciation", "get-pls", "--help"]));
}

// ============================================================================
// RAG Subcommands - 5
// ============================================================================

#[test]
fn h_rag_create() {
    assert!(ok(&["rag", "create", "--help"]));
}
#[test]
fn h_rag_status() {
    assert!(ok(&["rag", "status", "--help"]));
}
#[test]
fn h_rag_delete() {
    assert!(ok(&["rag", "delete", "--help"]));
}
#[test]
fn h_rag_rebuild() {
    assert!(ok(&["rag", "rebuild", "--help"]));
}
#[test]
fn h_rag_index_status() {
    assert!(ok(&["rag", "index-status", "--help"]));
}

// ============================================================================
// Workspace Subcommands - 10
// ============================================================================

#[test]
fn h_workspace_info() {
    assert!(ok(&["workspace", "info", "--help"]));
}
#[test]
fn h_workspace_members() {
    assert!(ok(&["workspace", "members", "--help"]));
}
#[test]
fn h_workspace_invites() {
    assert!(ok(&["workspace", "invites", "--help"]));
}
#[test]
fn h_workspace_invite() {
    assert!(ok(&["workspace", "invite", "--help"]));
}
#[test]
fn h_workspace_revoke() {
    assert!(ok(&["workspace", "revoke", "--help"]));
}
#[test]
fn h_workspace_api_keys() {
    assert!(ok(&["workspace", "api-keys", "--help"]));
}
#[test]
fn h_workspace_secrets() {
    assert!(ok(&["workspace", "secrets", "--help"]));
}
#[test]
fn h_workspace_add_secret() {
    assert!(ok(&["workspace", "add-secret", "--help"]));
}
#[test]
fn h_workspace_delete_secret() {
    assert!(ok(&["workspace", "delete-secret", "--help"]));
}
#[test]
fn h_workspace_share() {
    assert!(ok(&["workspace", "share", "--help"]));
}

// ============================================================================
// Phone Subcommands - 6
// ============================================================================

#[test]
fn h_phone_list() {
    assert!(ok(&["phone", "list", "--help"]));
}
#[test]
fn h_phone_get() {
    assert!(ok(&["phone", "get", "--help"]));
}
#[test]
fn h_phone_import() {
    assert!(ok(&["phone", "import", "--help"]));
}
#[test]
fn h_phone_update() {
    assert!(ok(&["phone", "update", "--help"]));
}
#[test]
fn h_phone_delete() {
    assert!(ok(&["phone", "delete", "--help"]));
}
#[test]
fn h_phone_test() {
    assert!(ok(&["phone", "test", "--help"]));
}

// ============================================================================
// Conversation Subcommands - 7
// ============================================================================

#[test]
fn h_converse_chat() {
    assert!(ok(&["converse", "chat", "--help"]));
}
#[test]
fn h_converse_list() {
    assert!(ok(&["converse", "list", "--help"]));
}
#[test]
fn h_converse_get() {
    assert!(ok(&["converse", "get", "--help"]));
}
#[test]
fn h_converse_signed_url() {
    assert!(ok(&["converse", "signed-url", "--help"]));
}
#[test]
fn h_converse_token() {
    assert!(ok(&["converse", "token", "--help"]));
}
#[test]
fn h_converse_delete() {
    assert!(ok(&["converse", "delete", "--help"]));
}
#[test]
fn h_converse_audio() {
    assert!(ok(&["converse", "audio", "--help"]));
}

// ============================================================================
// Agent Subcommands - 13
// ============================================================================

#[test]
fn h_agent_list() {
    assert!(ok(&["agent", "list", "--help"]));
}
#[test]
fn h_agent_summaries() {
    assert!(ok(&["agent", "summaries", "--help"]));
}
#[test]
fn h_agent_get() {
    assert!(ok(&["agent", "get", "--help"]));
}
#[test]
fn h_agent_create() {
    assert!(ok(&["agent", "create", "--help"]));
}
#[test]
fn h_agent_update() {
    assert!(ok(&["agent", "update", "--help"]));
}
#[test]
fn h_agent_delete() {
    assert!(ok(&["agent", "delete", "--help"]));
}
#[test]
fn h_agent_link() {
    assert!(ok(&["agent", "link", "--help"]));
}
#[test]
fn h_agent_duplicate() {
    assert!(ok(&["agent", "duplicate", "--help"]));
}
#[test]
fn h_agent_branches() {
    assert!(ok(&["agent", "branches", "--help"]));
}
#[test]
fn h_agent_rename_branch() {
    assert!(ok(&["agent", "rename-branch", "--help"]));
}
#[test]
fn h_agent_batch_list() {
    assert!(ok(&["agent", "batch-list", "--help"]));
}
#[test]
fn h_agent_batch_status() {
    assert!(ok(&["agent", "batch-status", "--help"]));
}
#[test]
fn h_agent_batch_delete() {
    assert!(ok(&["agent", "batch-delete", "--help"]));
}

// ============================================================================
// Projects Subcommands - 6
// ============================================================================

#[test]
fn h_projects_list() {
    assert!(ok(&["projects", "list", "--help"]));
}
#[test]
fn h_projects_get() {
    assert!(ok(&["projects", "get", "--help"]));
}
#[test]
fn h_projects_delete() {
    assert!(ok(&["projects", "delete", "--help"]));
}
#[test]
fn h_projects_convert() {
    assert!(ok(&["projects", "convert", "--help"]));
}
#[test]
fn h_projects_snapshots() {
    assert!(ok(&["projects", "snapshots", "--help"]));
}
#[test]
fn h_projects_audio() {
    assert!(ok(&["projects", "audio", "--help"]));
}

// ============================================================================
// Audio Native Subcommands - 3
// ============================================================================

#[test]
fn h_audio_native_list() {
    assert!(ok(&["audio-native", "list", "--help"]));
}
#[test]
fn h_audio_native_get() {
    assert!(ok(&["audio-native", "get", "--help"]));
}
#[test]
fn h_audio_native_create() {
    assert!(ok(&["audio-native", "create", "--help"]));
}

// ============================================================================
// Knowledge Subcommands - 6
// ============================================================================

#[test]
fn h_knowledge_list() {
    assert!(ok(&["knowledge", "list", "--help"]));
}
#[test]
fn h_knowledge_add_from_url() {
    assert!(ok(&["knowledge", "add-from-url", "--help"]));
}
#[test]
fn h_knowledge_add_from_text() {
    assert!(ok(&["knowledge", "add-from-text", "--help"]));
}
#[test]
fn h_knowledge_add_from_file() {
    assert!(ok(&["knowledge", "add-from-file", "--help"]));
}
#[test]
fn h_knowledge_get() {
    assert!(ok(&["knowledge", "get", "--help"]));
}
#[test]
fn h_knowledge_delete() {
    assert!(ok(&["knowledge", "delete", "--help"]));
}

// ============================================================================
// Music Subcommands - 3
// ============================================================================

#[test]
fn h_music_generate() {
    assert!(ok(&["music", "generate", "--help"]));
}
#[test]
fn h_music_list() {
    assert!(ok(&["music", "list", "--help"]));
}
#[test]
fn h_music_get() {
    assert!(ok(&["music", "get", "--help"]));
}
#[test]
fn h_music_download() {
    assert!(ok(&["music", "download", "--help"]));
}
#[test]
fn h_music_delete() {
    assert!(ok(&["music", "delete", "--help"]));
}

// ============================================================================
// Webhook Subcommands - 3
// ============================================================================

#[test]
fn h_webhook_list() {
    assert!(ok(&["webhook", "list", "--help"]));
}
#[test]
fn h_webhook_create() {
    assert!(ok(&["webhook", "create", "--help"]));
}
#[test]
fn h_webhook_delete() {
    assert!(ok(&["webhook", "delete", "--help"]));
}

// ============================================================================
// Library Subcommands - 5
// ============================================================================

#[test]
fn h_library_list() {
    assert!(ok(&["library", "list", "--help"]));
}
#[test]
fn h_library_saved() {
    assert!(ok(&["library", "saved", "--help"]));
}
#[test]
fn h_library_add() {
    assert!(ok(&["library", "add", "--help"]));
}
#[test]
fn h_library_collections() {
    assert!(ok(&["library", "collections", "--help"]));
}
#[test]
fn h_library_collection_voices() {
    assert!(ok(&["library", "collection-voices", "--help"]));
}

// ============================================================================
// Samples Subcommands - 2
// ============================================================================

#[test]
fn h_samples_list() {
    assert!(ok(&["samples", "list", "--help"]));
}
#[test]
fn h_samples_delete() {
    assert!(ok(&["samples", "delete", "--help"]));
}

// ============================================================================
// Tools Subcommands - 3
// ============================================================================

#[test]
fn h_tools_list() {
    assert!(ok(&["tools", "list", "--help"]));
}
#[test]
fn h_tools_get() {
    assert!(ok(&["tools", "get", "--help"]));
}
#[test]
fn h_tools_delete() {
    assert!(ok(&["tools", "delete", "--help"]));
}

// ============================================================================
// Dialogue - 1
// ============================================================================

#[test]
fn h_dialogue_cmd() {
    assert!(ok(&["dialogue", "--help"]));
}

// ============================================================================
// User - 2
// ============================================================================

#[test]
fn h_user_info() {
    assert!(ok(&["user", "info", "--help"]));
}
#[test]
fn h_user_subscription() {
    assert!(ok(&["user", "subscription", "--help"]));
}

// ============================================================================
// Config - 3
// ============================================================================

#[test]
fn h_config_show() {
    assert!(ok(&["config", "show", "--help"]));
}
#[test]
fn h_config_set() {
    assert!(ok(&["config", "set", "--help"]));
}
#[test]
fn h_config_unset() {
    assert!(ok(&["config", "unset", "--help"]));
}

// ============================================================================
// Error Cases - Missing Required Arguments
// ============================================================================

#[test]
fn e_tts() {
    assert!(fail(&["tts"]));
}
#[test]
fn e_stt() {
    assert!(fail(&["stt"]));
}
#[test]
fn e_voice_get() {
    assert!(fail(&["voice", "get"]));
}
#[test]
fn e_voice_clone() {
    assert!(fail(&["voice", "clone", "--name", "test"]));
}
#[test]
fn e_history_get() {
    assert!(fail(&["history", "get"]));
}
#[test]
fn e_history_feedback() {
    assert!(fail(&["history", "feedback"]));
}
#[test]
fn e_pronunciation_delete() {
    assert!(fail(&["pronunciation", "delete"]));
}
#[test]
fn e_rag_create() {
    assert!(fail(&["rag", "create"]));
}
#[test]
fn e_workspace_invite() {
    assert!(fail(&["workspace", "invite"]));
}
#[test]
fn e_phone_import() {
    assert!(fail(&["phone", "import"]));
}
#[test]
fn e_converse_chat() {
    assert!(fail(&["converse", "chat"]));
}
#[test]
fn e_agent_get() {
    assert!(fail(&["agent", "get"]));
}
#[test]
fn e_projects_get() {
    assert!(fail(&["projects", "get"]));
}
#[test]
fn e_knowledge_get() {
    assert!(fail(&["knowledge", "get"]));
}
#[test]
fn e_music_get() {
    assert!(fail(&["music", "get"]));
}
#[test]
fn e_webhook_delete() {
    assert!(fail(&["webhook", "delete"]));
}
#[test]
fn e_samples_delete() {
    assert!(fail(&["samples", "delete"]));
}
#[test]
fn e_audio_native_get() {
    assert!(fail(&["audio-native", "get"]));
}
#[test]
fn e_invalid() {
    assert!(fail(&["nonexistent"]));
}

// ============================================================================
// Global Options
// ============================================================================

#[test]
fn g_help() {
    assert!(ok(&["--help"]));
}
#[test]
fn g_version() {
    assert!(ok(&["--version"]));
}
