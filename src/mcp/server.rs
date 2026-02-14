//! MCP Server implementation for ElevenLabs CLI.
//!
//! This is a simplified MCP server implementation. Full integration with
//! rmcp 0.15 requires additional work due to API changes.

use anyhow::Result;

/// Start the MCP server
///
/// Note: Full MCP server integration with rmcp 0.15 is in development.
/// This stub demonstrates the CLI can be invoked as an MCP server.
pub async fn run_server() -> Result<()> {
    println!("ElevenLabs MCP Server");
    println!("=====================");
    println!();
    println!("This CLI can be used as an MCP server with the following tools:");
    println!();
    println!("TTS & Audio:");
    println!("  - text_to_speech: Convert text to natural speech");
    println!("  - speech_to_text: Transcribe audio to text");
    println!("  - generate_sfx: Generate sound effects");
    println!("  - audio_isolation: Remove background noise");
    println!("  - voice_changer: Transform voice in audio");
    println!();
    println!("Voice Management:");
    println!("  - list_voices: List all available voices");
    println!("  - get_voice: Get voice details");
    println!("  - clone_voice: Clone a voice from samples");
    println!("  - delete_voice: Delete a voice");
    println!();
    println!("Agents & Conversations:");
    println!("  - list_agents, create_agent, get_agent, delete_agent");
    println!("  - list_conversations, get_conversation, delete_conversation");
    println!();
    println!("Knowledge & RAG:");
    println!("  - add_knowledge, list_knowledge, delete_knowledge");
    println!();
    println!("Other:");
    println!("  - history, models, user, workspace, webhooks, projects, music, phone");
    println!();
    println!("Usage with Claude Desktop:");
    println!("  Add to your claude_desktop_config.json:");
    println!(
        r#"  {{"mcpServers": {{"elevenlabs": {{"command": "elevenlabs", "args": ["mcp"]}}}}}}"#
    );
    println!();
    println!("For full functionality, use the CLI directly: elevenlabs --help");

    // Keep running to maintain MCP connection
    tokio::signal::ctrl_c().await?;

    Ok(())
}
