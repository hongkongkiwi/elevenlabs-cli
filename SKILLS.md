# ElevenLabs CLI Skills

> **⚠️ Unofficial CLI**: This is an independent, community-built CLI client. It is not officially released by ElevenLabs.

A comprehensive command-line interface for the ElevenLabs AI audio platform with 100% SDK coverage.

> **For AI Agents**: This CLI exposes 80+ MCP tools covering all ElevenLabs functionality. Use `elevenlabs mcp` to start the MCP server.

## MCP Server (Model Context Protocol)

The CLI can run as an MCP server, exposing all ElevenLabs functionality to AI assistants.

### Installation with MCP Support

```bash
# Build with MCP feature
cargo install --path . --features mcp
```

### Starting the MCP Server

```bash
# Run as stdio MCP server (for AI assistants)
elevenlabs mcp

# Enable only specific tools
elevenlabs mcp --enable-tools tts,stt,voice

# Disable specific tools
elevenlabs mcp --disable-tools agents,phone

# Disable administrative operations
elevenlabs mcp --disable-admin

# Disable only destructive operations
elevenlabs mcp --disable-destructive

# Read-only mode
elevenlabs mcp --read-only
```

### MCP Tool Filtering

Control which tools are available in the MCP server:

- `--enable-tools`: Comma-separated list of tools to enable
- `--disable-tools`: Comma-separated list of tools to disable
- `--disable-admin`: Disable all delete, create, update operations
- `--disable-destructive`: Disable only delete operations
- `--read-only`: Read-only mode (same as --disable-admin)

This is useful for:
- Restricting AI assistants to read-only operations
- Limiting access to sensitive features
- Reducing the tool set for simpler AI interactions

### MCP Configuration for AI Clients

#### Claude Desktop
Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

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

#### OpenCode / Cursor / Other MCP Clients

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

### Available MCP Tools (80+ tools)

#### TTS & Audio Generation
| Tool | Description | Parameters |
|------|-------------|------------|
| `text_to_speech` | Convert text to natural speech | text (required), voice, model |
| `speech_to_text` | Transcribe audio to text | file (required), model, language, diarize |
| `generate_sfx` | Generate sound effects | text (required), duration |
| `audio_isolation` | Remove background noise | file (required) |
| `voice_changer` | Transform voice in audio | file (required), voice (required) |

#### Voice Management
| Tool | Description | Parameters |
|------|-------------|------------|
| `list_voices` | List all voices | detailed |
| `get_voice` | Get voice details | voice_id (required) |
| `delete_voice` | Delete a voice | voice_id (required) |
| `clone_voice` | Clone from samples | name (required), samples (required) |
| `voice_settings` | Get voice settings | voice_id (required) |
| `edit_voice_settings` | Edit settings | voice_id (required), stability, similarity_boost, style |
| `create_voice_design` | Create voice from text | text (required), voice_settings |
| `get_voice_design` | Get design status | design_id (required) |
| `start_voice_fine_tune` | Start fine-tuning | voice_id (required) |
| `get_voice_fine_tune_status` | Get fine-tune status | fine_tune_id (required) |
| `cancel_voice_fine_tune` | Cancel fine-tune | fine_tune_id (required) |
| `share_voice` | Share voice publicly | voice_id (required) |
| `get_similar_voices` | Find similar voices | voice_id (required) |

#### Dubbing
| Tool | Description | Parameters |
|------|-------------|------------|
| `create_dubbing` | Create dubbing project | file (required), source_lang (required), target_lang (required) |
| `get_dubbing_status` | Get dubbing status | dubbing_id (required) |
| `delete_dubbing` | Delete dubbing | dubbing_id (required) |

#### History
| Tool | Description | Parameters |
|------|-------------|------------|
| `list_history` | List history | limit |
| `get_history_item` | Get history item | history_item_id (required) |
| `delete_history_item` | Delete item | history_item_id (required) |
| `history_feedback` | Submit feedback | history_item_id (required), thumbs_up (required) |
| `download_history` | Download audio | history_item_id (required) |

#### Agents
| Tool | Description | Parameters |
|------|-------------|------------|
| `list_agents` | List agents | limit |
| `get_agent_summaries` | Get agent summaries | - |
| `create_agent` | Create agent | name (required) |
| `get_agent` | Get agent | agent_id (required) |
| `update_agent` | Update agent | agent_id (required) |
| `delete_agent` | Delete agent | agent_id (required) |
| `agent_branches` | List branches | agent_id (required) |
| `batch_list` | List batch jobs | - |

#### Conversation
| Tool | Description | Parameters |
|------|-------------|------------|
| `converse_chat` | Chat with agent | agent_id (required), message (required) |
| `list_conversations` | List conversations | - |
| `get_conversation` | Get conversation | conversation_id (required) |
| `get_signed_url` | Get signed URL | conversation_id (required) |
| `get_conversation_token` | Get token | agent_id (required) |
| `delete_conversation` | Delete conversation | conversation_id (required) |
| `get_conversation_audio` | Get audio | conversation_id (required) |

#### Knowledge & RAG
| Tool | Description | Parameters |
|------|-------------|------------|
| `list_knowledge` | List documents | limit |
| `add_knowledge` | Add document | source_type (required), name (required), content, url, file |
| `delete_knowledge` | Delete document | document_id (required) |
| `create_rag` | Create RAG | name (required) |
| `get_rag_status` | Get RAG status | rag_id (required) |
| `delete_rag` | Delete RAG | rag_id (required) |
| `rebuild_rag` | Rebuild RAG | rag_id (required) |
| `get_rag_index_status` | Get index status | rag_id (required) |

#### Projects
| Tool | Description | Parameters |
|------|-------------|------------|
| `list_projects` | List projects | - |
| `get_project` | Get project | project_id (required) |
| `delete_project` | Delete project | project_id (required) |
| `convert_project` | Convert project | project_id (required), target_format (required) |
| `list_project_snapshots` | List snapshots | project_id (required) |
| `get_project_audio` | Get audio | project_id (required) |

#### Music
| Tool | Description | Parameters |
|------|-------------|------------|
| `generate_music` | Generate music | prompt (required) |
| `list_music` | List music | limit |
| `get_music` | Get music | music_id (required) |
| `download_music` | Download music | music_id (required) |
| `delete_music` | Delete music | music_id (required) |

#### Phone
| Tool | Description | Parameters |
|------|-------------|------------|
| `list_phones` | List phones | - |
| `get_phone` | Get phone | phone_id (required) |
| `import_phone` | Import phone | number (required), provider |
| `update_phone` | Update phone | phone_id (required) |
| `delete_phone` | Delete phone | phone_id (required) |
| `test_phone_call` | Test call | phone_id (required) |

#### Other Tools
- **User**: `get_user_info`, `get_user_subscription`
- **Models**: `list_models`, `get_model_rates`
- **Usage**: `get_usage`
- **Webhooks**: `list_webhooks`, `create_webhook`, `delete_webhook`
- **Library**: `list_library_voices`, `list_library_collections`
- **Pronunciation**: `list_pronunciations`, `add_pronunciation`, `delete_pronunciation`, `list_pronunciation_rules`, `add_pronunciation_rules`, `remove_pronunciation_rules`, `get_pronunciation_pls`
- **Workspace**: `workspace_info`, `list_workspace_members`, `list_workspace_invites`, `invite_workspace_member`, `revoke_workspace_invite`, `list_workspace_api_keys`, `list_secrets`, `add_secret`, `delete_secret`, `share_workspace`
- **Samples**: `list_samples`, `delete_sample`
- **Tools**: `list_tools`, `get_tool`, `delete_tool`
- **Audio Native**: `list_audio_native`, `get_audio_native`, `create_audio_native`

## Core Skills

### Text-to-Speech
- Generate speech from text with 100+ voices
- Support for multiple models (eleven_multilingual_v2, eleven_v3, eleven_flash_v2_5, etc.)
- Voice settings (stability, similarity boost, style, speaker boost)
- Streaming TTS with timestamps
- TTS with character-level timestamps
- Output formats: mp3, wav, pcm, opus

### Speech-to-Text
- Transcribe audio with speaker diarization
- Automatic language detection
- Word-level timestamps
- Multiple models (scribe_v1, scribe_v1_base)
- Output formats: txt, srt, vtt, json

### Sound Effects
- Generate SFX from text descriptions
- Custom duration
- Prompt influence control

### Voice Changer
- Transform voice in audio files
- Multiple models

### Audio Isolation
- Remove background noise from audio
- Vocal isolation

### Dubbing
- Create dubbing projects (video/audio translation)
- Multi-language support
- Status tracking
- Download dubbed content

### Phone Numbers (Twilio/SIP)
- Import phone numbers
- List, get, update, delete
- Test calls
- Link to agents

### Webhooks
- Create, list, delete webhooks
- Event subscriptions

### Projects
- List, get, delete projects
- Convert projects
- Get snapshots
- Download project audio

### Audio Native
- Create embeddable audio players
- List, get projects
- Custom styling (colors, size)

### Music Generation
- Generate music from text prompts
- Status tracking
- Download generated music

### History
- List generation history
- Get, delete items
- Download audio
- Submit feedback (thumbs up/down)

### Usage
- Character usage statistics
- Date range filtering
- Breakdown by type

### User
- Account information
- Subscription details

### Models
- List available TTS/STT models
- Model rates and pricing

### Voice Library
- Browse library voices
- Filter by category, gender, age, language
- Search functionality
- Add to personal library
- Collections

### Samples
- List, delete voice samples
- Download sample audio

### Pronunciation
- List pronunciation dictionaries
- Add from PLS files
- Get, delete dictionaries
- List/add/remove rules
- Download PLS files

### Workspace Management
- Get workspace info
- List, invite, revoke members
- List, create, delete API keys
- List, add, delete secrets
- Share/unshare resources

## Usage

```bash
# Install
cargo install --path .

# Or with MCP support
cargo install --path . --features mcp

# Basic TTS
elevenlabs tts "Hello, world!"

# With options
elevenlabs tts "Hello" --voice Rachel --model eleven_v3 --output speech.mp3

# Transcribe
elevenlabs stt audio.mp3 --diarize

# Clone voice
elevenlabs voice clone --name "My Voice" --samples sample1.mp3,sample2.mp3

# List voices
elevenlabs voice list

# Run MCP server (with optional filtering)
elevenlabs mcp
# Or disable admin operations for safety
elevenlabs mcp --disable-admin
```

## Output Formats

- JSON output (`-j` flag)
- Table output (default)
- Text output

## Configuration

Config stored at `~/.config/elevenlabs-cli/config.toml`:
```toml
api_key = "your-api-key"
default_voice = "Brian"
default_model = "eleven_multilingual_v2"
default_output_format = "mp3_44100_128"
```

## AI Agent Usage

For AI agents (Claude, OpenCode, Cursor, etc.), use the MCP protocol:

```bash
# Start MCP server
elevenlabs mcp
```

The AI can then call any of the 80+ tools directly through the MCP protocol.
