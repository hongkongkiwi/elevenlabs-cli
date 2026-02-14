# ElevenLabs CLI - Comprehensive Code Refactoring Specification

## Status: COMPLETED ✓

## Summary of Changes

### New Modules Created

1. **`src/cli/`** - Split CLI definitions (26 files)
   - Organized by domain (tts, voice, agent, etc.)
   - Replaced monolithic `cli.rs` (816 lines)

2. **`src/output/`** - Output formatting utilities
   - `text.rs` - print_success, print_error, print_info, print_warning
   - `table.rs` - Table creation and formatting helpers

3. **`src/validation/`** - Input validation utilities
   - `voice_settings.rs` - Voice settings range validation

4. **`src/client/`** - API client abstractions
   - `api.rs` - Unified ApiClient wrapper
   - `retry.rs` - Retry logic with exponential backoff

### Files Modified

- `src/main.rs` - Updated to use new cli module structure

### Files Removed

- `src/cli.rs` - Replaced by `src/cli/` directory

## Test Results

- All 22 tests pass
- Release build successful
- All CLI commands work correctly

---

## Executive Summary

Refactor the ElevenLabs CLI codebase to improve code organization, reduce duplication, standardize patterns, and enhance maintainability while preserving all existing functionality.

## 1. Current Issues Analysis

### 1.1 Structural Issues

1. **Monolithic CLI Definition** (`src/cli.rs` - 816 lines)
   - All CLI argument structs in single file
   - Hard to navigate and maintain
   - Mixes concerns (TTS, Voice, Agent, etc.)

2. **Inconsistent Command Patterns** (`src/commands/`)
   - Some commands use `elevenlabs_rs` SDK (tts.rs, voice.rs)
   - Others use raw `reqwest::Client` (agent.rs, conversation.rs)
   - Different error handling approaches
   - Varying use of utilities

3. **Utility Sprawl** (`src/utils.rs` - 311 lines)
   - Mix of concerns (I/O, formatting, retries)
   - Some functions unused (dead code)
   - Test module mixed with implementation

4. **Duplicated Code**
   - Voice settings validation repeated in tts.rs, voice.rs
   - API client creation scattered across files
   - Table creation patterns duplicated
   - Error handling patterns inconsistent

5. **Missing Abstractions**
   - No shared API client wrapper
   - No common response types
   - No standardized output formatting

### 1.2 Maintainability Issues

1. Adding new command requires changes in 3+ files
2. No clear module boundaries
3. Hard to test individual components
4. Difficult for new contributors to understand structure

## 2. Functional Requirements

### 2.1 Must Preserve
- All 30+ existing commands and subcommands
- All CLI flags and arguments (backward compatible)
- JSON output mode
- MCP server functionality
- Interactive mode
- Configuration system
- Shell completion generation

### 2.2 Must Improve
- Module organization (separation by domain)
- Code reuse (shared utilities and patterns)
- Consistency across command implementations
- Test organization
- Error handling patterns

## 3. Proposed Architecture

### 3.1 New Directory Structure

```
src/
├── main.rs              # Entry point (minimal)
├── lib.rs               # Library root
│
├── cli/                 # CLI definitions by domain
│   ├── mod.rs
│   ├── args.rs          # Global args (Cli struct)
│   ├── tts.rs           # TTS-related args
│   ├── stt.rs           # STT-related args
│   ├── voice.rs         # Voice management args
│   ├── audio.rs         # Audio processing args (isolation, sfx, voice-changer)
│   ├── agent.rs         # Agent/conversation args
│   ├── content.rs       # Content args (history, knowledge, projects)
│   ├── workspace.rs     # Workspace/config args
│   └── output.rs        # Output-related args (completions, interactive)
│
├── client/              # API client abstraction
│   ├── mod.rs
│   ├── elevenlabs.rs    # ElevenLabsClient wrapper
│   ├── http.rs          # HTTP utilities
│   └── retry.rs         # Retry logic
│
├── commands/            # Command implementations (reorganized)
│   ├── mod.rs
│   ├── tts/             # TTS commands
│   │   ├── mod.rs
│   │   ├── basic.rs     # Basic TTS
│   │   ├── timestamps.rs
│   │   └── stream.rs
│   ├── stt/             # STT commands
│   │   └── mod.rs
│   ├── voice/           # Voice management
│   │   ├── mod.rs
│   │   ├── clone.rs
│   │   └── settings.rs
│   ├── audio/           # Audio processing
│   │   ├── mod.rs
│   │   ├── isolation.rs
│   │   ├── sfx.rs
│   │   └── voice_changer.rs
│   ├── agent/           # Agent management
│   │   ├── mod.rs
│   │   ├── crud.rs      # Create/Read/Update/Delete
│   │   ├── branches.rs
│   │   └── batch.rs
│   ├── conversation/    # WebSocket conversation
│   │   └── mod.rs
│   ├── content/         # Content management
│   │   ├── mod.rs
│   │   ├── history.rs
│   │   ├── knowledge.rs
│   │   ├── projects.rs
│   │   └── rag.rs
│   ├── media/           # Media generation
│   │   ├── mod.rs
│   │   ├── dubbing.rs
│   │   ├── dialogue.rs
│   │   └── music.rs
│   ├── library/         # Voice library
│   │   └── mod.rs
│   ├── system/          # System/config
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── user.rs
│   │   ├── usage.rs
│   │   ├── models.rs
│   │   └── workspace.rs
│   ├── tools/           # Agent tools
│   │   └── mod.rs
│   ├── webhook/         # Webhooks
│   │   └── mod.rs
│   └── output/          # Output utilities
│       ├── mod.rs
│       ├── completions.rs
│       └── interactive.rs
│
├── output/              # Output formatting (new)
│   ├── mod.rs
│   ├── table.rs         # Table formatting utilities
│   ├── json.rs          # JSON output handling
│   └── text.rs          # Text/colored output
│
├── validation/          # Input validation (new)
│   ├── mod.rs
│   └── voice_settings.rs
│
├── config/              # Configuration (renamed from src/config.rs)
│   └── mod.rs
│
├── mcp/                 # MCP server (unchanged)
│   ├── mod.rs
│   └── handlers.rs
│
└── utils/               # General utilities (trimmed)
    ├── mod.rs
    ├── io.rs            # File I/O utilities
    └── format.rs        # Format conversion utilities
```

### 3.2 Key Abstractions

#### 3.2.1 Unified API Client

```rust
// src/client/mod.rs
pub struct ApiClient {
    elevenlabs: ElevenLabsClient,
    http: reqwest::Client,
    api_key: String,
}

impl ApiClient {
    pub fn new(api_key: &str) -> Self;
    pub fn elevenlabs(&self) -> &ElevenLabsClient;
    pub async fn request<T: DeserializeOwned>(&self, req: RequestBuilder) -> Result<T>;
}
```

#### 3.2.2 Voice Settings Validation

```rust
// src/validation/voice_settings.rs
pub struct VoiceSettingsValidator;

impl VoiceSettingsValidator {
    pub fn validate_stability(value: f32) -> Result<f32>;
    pub fn validate_similarity_boost(value: f32) -> Result<f32>;
    pub fn validate_style(value: f32) -> Result<f32>;
    pub fn validate_all(stability: Option<f32>, similarity: Option<f32>, style: Option<f32>) -> Result<()>;
}
```

#### 3.2.3 Output Formatters

```rust
// src/output/mod.rs
pub trait OutputFormat {
    fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String;
    fn format_success(msg: &str) -> String;
    fn format_error(msg: &str) -> String;
    fn format_info(msg: &str) -> String;
    fn format_warning(msg: &str) -> String;
}

pub struct TableOutput;
pub struct JsonOutput;
pub struct TextOutput;
```

### 3.3 Standard Command Pattern

Each command module should follow this pattern:

```rust
// src/commands/tts/mod.rs
mod basic;
mod timestamps;
mod stream;

pub use basic::execute as execute_basic;
pub use timestamps::execute as execute_timestamps;
pub use stream::execute as execute_stream;

use crate::cli::tts::{TextToSpeechArgs, TtsTimestampsArgs, TtsStreamArgs};
use crate::client::ApiClient;
use crate::output::{Output, TableOutput};

pub async fn execute(args: TextToSpeechArgs, client: &ApiClient, assume_yes: bool) -> Result<()> {
    // 1. Validate input
    // 2. Build request
    // 3. Call API
    // 4. Format and display output
}
```

## 4. Implementation Plan

### Phase 1: Foundation (No Breaking Changes)
1. Create new directory structure
2. Create abstraction modules (client, output, validation)
3. Extract shared utilities
4. Add module re-exports for compatibility

### Phase 2: CLI Definitions Migration
1. Split `cli.rs` into domain-specific files
2. Update imports in main.rs
3. Verify compilation

### Phase 3: Command Migration (by domain)
1. Audio commands (tts, stt, voice, etc.)
2. Agent commands
3. Content commands
4. System commands

### Phase 4: Cleanup
1. Remove dead code
2. Update tests
3. Update documentation
4. Final verification

## 5. Non-Functional Requirements

### 5.1 Performance
- No regression in command execution time
- Minimal increase in binary size

### 5.2 Maintainability
- Each module < 300 lines
- Clear separation of concerns
- Consistent naming conventions
- Comprehensive module documentation

### 5.3 Compatibility
- All existing CLI invocations work unchanged
- Config file format unchanged
- JSON output format unchanged

## 6. Out of Scope

- Adding new features or commands
- Changing CLI argument names or behavior
- Modifying MCP protocol implementation
- Changing configuration file format
- UI/UX changes to interactive mode
- Performance optimizations (unless incidental)
