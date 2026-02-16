# ElevenLabs CLI

> **⚠️ Unofficial CLI**: This is an independent, community-built CLI client. It is not officially released by ElevenLabs.

A comprehensive command-line interface for the ElevenLabs AI audio platform. Generate speech, transcribe audio, clone voices, manage agents, and more - all from your terminal.

> **Official Documentation**: This CLI wraps the [ElevenLabs API](https://elevenlabs.io/docs/api-reference). For detailed API documentation, visit the official [ElevenLabs Docs](https://elevenlabs.io/docs).

## Features

- **Text-to-Speech**: Convert text to natural speech with 100+ voices
- **Speech-to-Text**: Transcribe audio with speaker diarization and timestamps
- **Voice Cloning**: Clone voices from audio samples
- **Sound Effects**: Generate sound effects from text descriptions
- **Voice Changer**: Transform voice in audio files
- **Dubbing**: Translate and dub video/audio to other languages
- **Audio Isolation**: Remove background noise from audio
- **Agents**: Create and manage conversational AI agents
- **Dialogue**: Generate multi-voice dialogues
- **MCP Support**: Use as an MCP server for AI assistants

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/hongkongkiwi/elevenlabs-cli.git
cd elevenlabs-cli

# Build and install
cargo install --path .
```

### With MCP Support

```bash
cargo install --path . --features mcp
```

## Configuration

### API Key

Set your ElevenLabs API key via environment variable:

```bash
export ELEVENLABS_API_KEY="your-api-key-here"
```

> **Get your API key**: [ElevenLabs API Keys Settings](https://elevenlabs.io/app/settings/api-keys)

Or pass it with each command:

```bash
elevenlabs --api-key "your-api-key" tts "Hello, world!"
```

Or save it to config:

```bash
elevenlabs config set api_key "your-api-key"
```

### Config File

Config is stored at `~/.config/elevenlabs-cli/config.toml`:

```toml
api_key = "your-api-key"
default_voice = "Brian"
default_model = "eleven_multilingual_v2"
default_output_format = "mp3_44100_128"
```

## Usage

### Text-to-Speech

> **API Docs**: [Text-to-Speech API Reference](https://elevenlabs.io/docs/api-reference/text-to-speech)

```bash
# Basic usage
elevenlabs tts "Hello, world!"

# With specific voice and model
elevenlabs tts "Hello, world!" --voice Brian --model eleven_multilingual_v2

# Save to file
elevenlabs tts "Hello, world!" --output speech.mp3

# Read from file
elevenlabs tts --file input.txt --output speech.mp3

# With voice settings
elevenlabs tts "Hello, world!" --stability 0.5 --similarity-boost 0.75

# Play after generation (macOS)
elevenlabs tts "Hello, world!" --play
```

### Speech-to-Text

> **API Docs**: [Speech-to-Text API Reference](https://elevenlabs.io/docs/api-reference/speech-to-text)

```bash
# Transcribe audio file
elevenlabs stt audio.mp3

# With speaker diarization
elevenlabs stt audio.mp3 --diarize --num-speakers 2

# Output as SRT subtitles
elevenlabs stt audio.mp3 --format srt --output subtitles.srt

# With timestamps
elevenlabs stt audio.mp3 --timestamps word
```

### Voice Management

> **API Docs**: [Voices API Reference](https://elevenlabs.io/docs/api-reference/voices)

```bash
# List all voices
elevenlabs voice list

# Get voice details
elevenlabs voice get <voice-id>

# Clone a voice
elevenlabs voice clone --name "My Voice" --samples sample1.mp3,sample2.mp3

# Clone from directory
elevenlabs voice clone --name "My Voice" --samples-dir ./samples/

# Delete a voice
elevenlabs voice delete <voice-id>
```

### Sound Effects

> **API Docs**: [Sound Effects API Reference](https://elevenlabs.io/docs/api-reference/sound-effects)

```bash
# Generate sound effect
elevenlabs sfx "A thunderstorm with heavy rain"

# With specific duration
elevenlabs sfx "A car engine revving" --duration 5.0

# Save to file
elevenlabs sfx "Door bell ringing" --output doorbell.mp3
```

### Voice Changer

> **API Docs**: [Voice Changer API Reference](https://elevenlabs.io/docs/api-reference/voice-changer)

```bash
# Transform voice in audio file
elevenlabs voice-changer input.mp3 --voice Brian --output transformed.mp3
```

### Audio Isolation

> **API Docs**: [Audio Isolation API Reference](https://elevenlabs.io/docs/api-reference/audio-isolation)

```bash
# Remove background noise
elevenlabs isolate noisy_audio.mp3 --output clean_audio.mp3
```

### Dubbing

> **API Docs**: [Dubbing API Reference](https://elevenlabs.io/docs/api-reference/dubbing)

```bash
# Create dubbing project
elevenlabs dub create --file video.mp4 --source-lang en --target-lang es

# Check status
elevenlabs dub status <dubbing-id>

# Download dubbed file
elevenlabs dub download <dubbing-id> --output dubbed.mp4
```

### Agents

> **API Docs**: [Conversational AI API Reference](https://elevenlabs.io/docs/api-reference/conversational-ai)

```bash
# List agents
elevenlabs agent list

# Create agent
elevenlabs agent create --name "Support Bot" --voice-id <voice-id> --first-message "Hello! How can I help?"

# Get agent details
elevenlabs agent get <agent-id>

# List agent summaries (lightweight)
elevenlabs agent summaries

# Manage branches
elevenlabs agent branches <agent-id>
elevenlabs agent rename-branch <agent-id> <branch-id> --name "production"

# Batch calls
elevenlabs agent batch-list
elevenlabs agent batch-status <batch-id>
elevenlabs agent batch-delete <batch-id>
```

### Dialogue (Multi-Voice)

> **API Docs**: [Text-to-Dialogue API Reference](https://elevenlabs.io/docs/api-reference/text-to-dialogue)

```bash
# Create dialogue with multiple voices
elevenlabs dialogue --inputs "Hello!:voice1,Hi there!:voice2,How are you?:voice1"

# With specific model
elevenlabs dialogue --inputs "text1:voice1,text2:voice2" --model eleven_v3

# Save to file
elevenlabs dialogue --inputs "..." --output dialogue.mp3
```

### Knowledge Base

> **API Docs**: [Knowledge Base API Reference](https://elevenlabs.io/docs/api-reference/knowledge-base)

```bash
# List documents
elevenlabs knowledge list

# Add from URL
elevenlabs knowledge add-from-url --url https://example.com/doc --name "Documentation"

# Add from text
elevenlabs knowledge add-from-text --text "Content here" --name "My Doc"

# Add from file
elevenlabs knowledge add-from-file --file document.pdf --name "PDF Doc"

# Delete document
elevenlabs knowledge delete <document-id>
```

### Webhooks

> **API Docs**: [Webhooks API Reference](https://elevenlabs.io/docs/api-reference/webhooks)

```bash
# List webhooks
elevenlabs webhook list

# Create webhook
elevenlabs webhook create --name "My Webhook" --url https://example.com/webhook --events voice.created,voice.deleted
```

### History

> **API Docs**: [History API Reference](https://elevenlabs.io/docs/api-reference/history)

```bash
# List generation history
elevenlabs history list

# Get history item details
elevenlabs history get <history-item-id>

# Download audio from history
elevenlabs history download <history-item-id> --output audio.mp3

# Delete history item
elevenlabs history delete <history-item-id>
```

### Usage Statistics

> **API Docs**: [Usage API Reference](https://elevenlabs.io/docs/api-reference/usage)

```bash
# Get usage stats
elevenlabs usage stats

# With date range
elevenlabs usage stats --start 1704067200 --end 1706745600

# With breakdown
elevenlabs usage stats --breakdown voice
```

### Workspace

> **API Docs**: [Workspace API Reference](https://elevenlabs.io/docs/api-reference/workspace)

```bash
# Get workspace info
elevenlabs workspace info

# List pending invites
elevenlabs workspace invites

# Invite member
elevenlabs workspace invite user@example.com --role editor

# Revoke invite
elevenlabs workspace revoke user@example.com
```

## MCP Server

The CLI can run as an MCP (Model Context Protocol) server, exposing all ElevenLabs functionality to AI assistants like Claude, OpenCode, Cursor, and others.

> **Learn more**: [Model Context Protocol](https://modelcontextprotocol.io/)

```bash
# Run as MCP server (requires --features mcp)
elevenlabs mcp

# Enable only specific tools
elevenlabs mcp --enable-tools tts,stt,voice

# Disable specific tools
elevenlabs mcp --disable-tools agents,phone

# Disable all administrative/destructive operations (delete, create, update)
elevenlabs mcp --disable-admin
```

This allows AI assistants to:
- Generate speech and sound effects
- Transcribe audio
- Manage voices and agents
- Access all ElevenLabs API features (80+ tools)

### MCP Tool Filtering

You can control which tools are available in the MCP server:

- `--enable-tools`: Comma-separated list of tools to enable (others disabled)
- `--disable-tools`: Comma-separated list of tools to disable
- `--disable-admin`: Disable all administrative/destructive operations (delete, create, update)

Administrative tools blocked by `--disable-admin`:
- Voice: delete_voice, clone_voice, edit_voice_settings, share_voice
- Agents: create_agent, update_agent, delete_agent, duplicate_agent
- Knowledge: add_knowledge, delete_knowledge, create_rag, delete_rag, rebuild_rag
- And many more...

### MCP Configuration

Add to your MCP client configuration:

```json
{
  "mcpServers": {
    "elevenlabs": {
      "command": "elevenlabs",
      "args": ["mcp"],
      "env": {
        "ELEVENLABS_API_KEY": "your-api-key"
      }
    }
  }
}
```

### Available MCP Tools

The MCP server exposes 80+ tools organized by category:

| Category | Tools |
|----------|-------|
| **TTS & Audio** | `text_to_speech`, `speech_to_text`, `generate_sfx`, `audio_isolation`, `voice_changer` |
| **Voice Management** | `list_voices`, `get_voice`, `delete_voice`, `clone_voice`, `voice_settings`, `edit_voice_settings`, `create_voice_design`, `get_voice_design`, `start_voice_fine_tune`, `get_voice_fine_tune_status`, `cancel_voice_fine_tune`, `share_voice`, `get_similar_voices` |
| **Dubbing** | `create_dubbing`, `get_dubbing_status`, `delete_dubbing` |
| **History** | `list_history`, `get_history_item`, `delete_history_item`, `history_feedback`, `download_history` |
| **Agents** | `list_agents`, `get_agent_summaries`, `create_agent`, `get_agent`, `update_agent`, `delete_agent`, `agent_branches`, `batch_list` |
| **Conversation** | `converse_chat`, `list_conversations`, `get_conversation`, `get_signed_url`, `get_conversation_token`, `delete_conversation`, `get_conversation_audio` |
| **Knowledge & RAG** | `list_knowledge`, `add_knowledge`, `delete_knowledge`, `create_rag`, `get_rag_status`, `delete_rag`, `rebuild_rag`, `get_rag_index_status` |
| **Projects** | `list_projects`, `get_project`, `delete_project`, `convert_project`, `list_project_snapshots`, `get_project_audio` |
| **Music** | `generate_music`, `list_music`, `get_music`, `download_music`, `delete_music` |
| **Phone** | `list_phones`, `get_phone`, `import_phone`, `update_phone`, `delete_phone`, `test_phone_call` |
| **Workspace** | `workspace_info`, `list_workspace_members`, `list_workspace_invites`, `invite_workspace_member`, `revoke_workspace_invite`, `list_workspace_api_keys`, `list_secrets`, `add_secret`, `delete_secret`, `share_workspace` |
| **Other** | `list_models`, `get_model_rates`, `get_usage`, `get_user_info`, `get_user_subscription`, `list_webhooks`, `create_webhook`, `delete_webhook`, `list_library_voices`, `list_library_collections`, `list_pronunciations`, `add_pronunciation`, `delete_pronunciation`, `list_samples`, `delete_sample`, `list_tools`, `get_tool`, `delete_tool`, `list_audio_native`, `get_audio_native`, `create_audio_native` |

## Output Formats

Supported audio output formats:

| Format | Description | Docs |
|--------|-------------|------|
| `mp3_44100_128` | MP3 44.1kHz 128kbps (default) | [Audio Formats](https://elevenlabs.io/docs/api-reference/text-to-speech#output-formats) |
| `mp3_44100_192` | MP3 44.1kHz 192kbps | |
| `pcm_16000` | PCM 16kHz | |
| `pcm_22050` | PCM 22.05kHz | |
| `pcm_44100` | PCM 44.1kHz | |
| `wav_8000` | WAV 8kHz | |
| `wav_16000` | WAV 16kHz | |
| `wav_44100` | WAV 44.1kHz | |
| `opus_48000_64` | Opus 48kHz 64kbps | |
| `opus_48000_128` | Opus 48kHz 128kbps | |
| `ulaw_8000` | μ-law 8kHz | |

## Models

| Model | Use Case | Docs |
|-------|----------|------|
| `eleven_multilingual_v2` | Best quality, 29 languages | [Models Guide](https://elevenlabs.io/docs/models) |
| `eleven_flash_v2_5` | Lowest latency | |
| `eleven_turbo_v2_5` | Balanced quality and speed | |
| `eleven_v3` | Expressive, emotional speech | |
| `scribe_v1` | Speech-to-text transcription | [STT Models](https://elevenlabs.io/docs/speech-to-text/models) |
| `scribe_v1_base` | Faster, lower cost STT | |

## Shell Completion

Generate shell completions:

```bash
# Bash
elevenlabs completions bash > /etc/bash_completion.d/elevenlabs

# Zsh
elevenlabs completions zsh > "${fpath[1]}/_elevenlabs"

# Fish
elevenlabs completions fish > ~/.config/fish/completions/elevenlabs.fish

# PowerShell
elevenlabs completions powershell > ~/.config/powershell/Microsoft.PowerShell_profile.ps1
```

## Interactive Mode

Run in interactive REPL mode:

```bash
elevenlabs interactive
```

Available commands in interactive mode:
- `tts <text>` - Text to speech
- `stt <file>` - Speech to text
- `voices` - List voices
- `models` - List models
- `user` - User info
- `exit` - Exit interactive mode

## JSON Output

For scripting, use JSON output mode:

```bash
elevenlabs -j voice list
elevenlabs --json stt audio.mp3 --format json
```

## Error Handling

The CLI includes automatic retry logic for transient failures:
- Retries up to 3 times with exponential backoff
- Handles 5xx server errors and rate limits (429)

> **Learn more**: [API Error Codes](https://elevenlabs.io/docs/api-reference/error-codes)

## Resources

| Resource | Link |
|----------|------|
| API Reference | https://elevenlabs.io/docs/api-reference |
| Documentation | https://elevenlabs.io/docs |
| API Keys | https://elevenlabs.io/app/settings/api-keys |
| Dashboard | https://elevenlabs.io/app |
| Voice Library | https://elevenlabs.io/app/voice-library |
| Changelog | https://elevenlabs.io/docs/changelog |
| Status Page | https://status.elevenlabs.io |

## License

MIT License - see [LICENSE](LICENSE) file.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

- [GitHub Issues](https://github.com/hongkongkiwi/elevenlabs-cli/issues)
- [ElevenLabs Documentation](https://elevenlabs.io/docs)
- [ElevenLabs API Reference](https://elevenlabs.io/docs/api-reference)
- [ElevenLabs Support](https://elevenlabs.io/support)
