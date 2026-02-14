# Implementation Plan: ElevenLabs CLI Refactoring

## Overview

This plan reorganizes the ElevenLabs CLI codebase to improve maintainability, reduce duplication, and standardize patterns. The refactoring is done incrementally to avoid breaking the build.

## Execution Strategy

- **Parallel execution**: Independent tasks can run in parallel
- **Incremental**: Each step maintains a working build
- **Verification**: Run `cargo check` and `cargo test` after each major change

---

## Phase 1: Foundation Layer (Create New Modules)

### Step 1.1: Create Output Module
**Files to create:**
- `src/output/mod.rs`
- `src/output/table.rs`
- `src/output/text.rs`
- `src/output/json.rs`

**Content:**
- Move `print_success`, `print_error`, `print_info`, `print_warning` to `output/text.rs`
- Extract table formatting patterns to `output/table.rs`
- Add JSON output handling to `output/json.rs`

### Step 1.2: Create Validation Module
**Files to create:**
- `src/validation/mod.rs`
- `src/validation/voice_settings.rs`

**Content:**
- Extract voice settings validation (stability, similarity_boost, style range checks)
- Create reusable validation functions

### Step 1.3: Create Client Module
**Files to create:**
- `src/client/mod.rs`
- `src/client/api.rs`
- `src/client/retry.rs`

**Content:**
- Wrap `ElevenLabsClient` and `reqwest::Client`
- Extract retry logic from utils.rs
- Create unified API client interface

---

## Phase 2: Split CLI Definitions

### Step 2.1: Create cli/ directory structure
**Files to create:**
- `src/cli/mod.rs` (exports all submodules)
- `src/cli/args.rs` (global Cli struct)

### Step 2.2: Split by domain (can parallelize)
**Files to create (independent):**
- `src/cli/tts.rs` - TextToSpeechArgs, TtsTimestampsArgs, TtsStreamArgs
- `src/cli/stt.rs` - SpeechToTextArgs
- `src/cli/voice.rs` - VoiceArgs, VoiceCommands
- `src/cli/audio.rs` - AudioIsolationArgs, SoundEffectsArgs, VoiceChangerArgs
- `src/cli/dubbing.rs` - DubbingArgs, DubbingCommands
- `src/cli/agent.rs` - AgentArgs, AgentCommands, ConversationArgs
- `src/cli/content.rs` - HistoryArgs, KnowledgeArgs, ProjectsArgs, RagArgs
- `src/cli/library.rs` - VoiceLibraryArgs, VoiceDesignArgs, SamplesArgs
- `src/cli/system.rs` - UserArgs, UsageArgs, ModelsArgs, WorkspaceArgs, ConfigArgs
- `src/cli/tools.rs` - ToolsArgs, WebhookArgs, PronunciationArgs
- `src/cli/media.rs` - DialogueArgs, MusicArgs
- `src/cli/native.rs` - AudioNativeArgs
- `src/cli/output.rs` - Completions, Interactive (handle in main.rs)

---

## Phase 3: Reorganize Commands

### Step 3.1: Create commands subdirectories
**Directories to create:**
- `src/commands/tts/`
- `src/commands/audio/`
- `src/commands/agent/`
- `src/commands/content/`
- `src/commands/system/`

### Step 3.2: Move and update command files
**Each move includes:**
1. Create new mod.rs for subdirectory
2. Move existing file to new location
3. Update imports to use new module paths
4. Update src/commands/mod.rs

### Step 3.3: Standardize command implementations
**For each command:**
1. Use shared `ApiClient` instead of direct client creation
2. Use `output` module for printing
3. Use `validation` module for input validation
4. Follow consistent error handling pattern

---

## Phase 4: Update Main Entry Point

### Step 4.1: Update main.rs imports
- Import from new `cli/` modules
- Import from new `client/` module
- Import from new `output/` module

### Step 4.2: Simplify main.rs
- Use new ApiClient
- Reduce boilerplate
- Maintain same command routing logic

---

## Phase 5: Cleanup

### Step 5.1: Remove old files (after migration verified)
- Delete original `src/cli.rs` (replaced by `src/cli/`)
- Clean up `src/utils.rs` (keep only truly generic utilities)

### Step 5.2: Update lib.rs if needed
- Ensure proper module exports
- Update any public API

### Step 5.3: Update documentation
- Update inline comments
- Add module-level documentation

---

## Verification Checkpoints

After each phase:
```bash
cargo check          # Must pass
cargo test           # Must pass
cargo clippy         # Should pass with no warnings
```

After complete refactoring:
```bash
cargo build --release
./target/release/elevenlabs --help
./target/release/elevenlabs tts --help
./target/release/elevenlabs voice list --help
```

---

## Risk Mitigation

1. **Git branches**: Work on feature branch, keep main stable
2. **Incremental commits**: Commit after each working state
3. **Backward compatibility**: All existing CLI invocations must work
4. **No new features**: Only reorganize, don't add functionality

---

## Estimated Tasks

| Task | Complexity | Dependencies |
|------|------------|--------------|
| Create output module | Low | None |
| Create validation module | Low | None |
| Create client module | Medium | None |
| Split CLI definitions | Medium | None |
| Reorganize commands | High | Client module |
| Update main.rs | Medium | All above |
| Cleanup | Low | All above |

## Parallel Execution Opportunities

The following can be done in parallel:
- Steps 1.1, 1.2, 1.3 (Foundation modules)
- All Step 2.2 subtasks (CLI splits)
- Multiple command reorganizations once structure is ready
