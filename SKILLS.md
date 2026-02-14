# ElevenLabs CLI Skills

A comprehensive command-line interface for the ElevenLabs AI audio platform with 100% SDK coverage.

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

### Voice Management
- List, get, delete voices
- Clone voices from audio samples
- Edit voice settings (stability, similarity, style)
- Voice fine-tuning (start, status, cancel)
- Edit voice name/description
- Share voice publicly
- Find similar voices

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

## Agent Skills

### Conversational AI
- Create, list, get, delete agents
- Agent configuration (voice, prompt, first message)
- Agent branches management
- Batch calls
- WebSocket conversations
- Conversation management (list, get, delete)
- Signed URLs and tokens

### Knowledge Base
- Add documents (URL, text, file)
- List, get, delete documents
- RAG index management
- Rebuild and status tracking

### Tools
- List, get, delete agent tools

## Communication Skills

### Phone Numbers (Twilio/SIP)
- Import phone numbers
- List, get, update, delete
- Test calls
- Link to agents

### Webhooks
- Create, list, delete webhooks
- Event subscriptions

## Project Skills

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

## Data Management

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

## Model Skills

### Models
- List available TTS/STT models
- Model rates and pricing

## Voice Library

### Shared Voices
- Browse library voices
- Filter by category, gender, age, language
- Search functionality
- Add to personal library

### Collections
- List collections
- Get voices in collection

### Samples
- List, delete voice samples
- Download sample audio

## Pronunciation

### Dictionaries
- List pronunciation dictionaries
- Add from PLS files
- Get, delete dictionaries
- List/add/remove rules
- Download PLS files

## Workspace Skills

### Workspace Management
- Get workspace info
- List, invite, revoke members
- List, create, delete API keys
- List, add, delete secrets
- Share/unshare resources

## Utility Skills

### Configuration
- Set/get configuration
- Default voice, model, output format

### Shell Completions
- Bash, Zsh, Fish, PowerShell

### Interactive Mode
- REPL for quick commands

### MCP Server
- Model Context Protocol server
- stdio transport for AI assistants

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

# Run MCP server
elevenlabs mcp
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
