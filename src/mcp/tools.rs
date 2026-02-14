//! Tool definitions for MCP server.
//!
//! This module provides tool schemas and handlers for all ElevenLabs API operations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_tts_model() -> String {
    "eleven_multilingual_v2".to_string()
}

fn default_output_format() -> String {
    "mp3_44100_128".to_string()
}

// ============================================================================
// TTS Tools
// ============================================================================

/// Convert text to speech
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TextToSpeechInput {
    /// The text to convert to speech
    pub text: String,
    /// Voice ID or name to use (e.g., "Brian", "Rachel", or a voice ID)
    pub voice: String,
    /// Model to use: eleven_multilingual_v2, eleven_flash_v2_5, eleven_turbo_v2, eleven_v3
    #[serde(default = "default_tts_model")]
    pub model: String,
    /// Output format: mp3_44100_128, mp3_44100_192, pcm_16000, etc.
    #[serde(default = "default_output_format")]
    pub output_format: String,
    /// Voice stability (0.0-1.0, higher = more stable)
    #[serde(default)]
    pub stability: Option<f32>,
    /// Voice similarity boost (0.0-1.0)
    #[serde(default)]
    pub similarity_boost: Option<f32>,
    /// Voice style (0.0-1.0)
    #[serde(default)]
    pub style: Option<f32>,
    /// Enable speaker boost
    #[serde(default)]
    pub speaker_boost: bool,
    /// Output file path (if not specified, returns base64 audio)
    #[serde(default)]
    pub output_file: Option<String>,
}

/// Result of text-to-speech conversion
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TextToSpeechOutput {
    /// Whether the operation was successful
    pub success: bool,
    /// Output file path (if saved to disk)
    pub output_file: Option<String>,
    /// Base64-encoded audio data (if not saved to disk)
    pub audio_base64: Option<String>,
    /// Audio duration in seconds
    pub duration_seconds: Option<f64>,
    /// Error message if unsuccessful
    pub error: Option<String>,
}

// ============================================================================
// Speech-to-Text Tools
// ============================================================================

/// Transcribe audio to text
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SpeechToTextInput {
    /// Path to the audio file to transcribe
    pub file: String,
    /// Model: scribe_v1, scribe_v1_base, scribe_v2
    #[serde(default = "default_stt_model")]
    pub model: String,
    /// Language code (auto-detected if not specified)
    #[serde(default)]
    pub language: Option<String>,
    /// Enable speaker diarization
    #[serde(default)]
    pub diarize: bool,
    /// Number of speakers (for diarization)
    #[serde(default)]
    pub num_speakers: Option<u32>,
    /// Timestamps granularity: none, word, character
    #[serde(default = "default_timestamps")]
    pub timestamps: String,
}

fn default_stt_model() -> String {
    "scribe_v1".to_string()
}

fn default_timestamps() -> String {
    "word".to_string()
}

/// Result of speech-to-text transcription
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SpeechToTextOutput {
    /// Whether the operation was successful
    pub success: bool,
    /// Transcribed text
    pub text: Option<String>,
    /// Detected language code
    pub language_code: Option<String>,
    /// Language detection confidence
    pub language_probability: Option<f64>,
    /// Word-level timestamps
    pub words: Option<Vec<WordTimestamp>>,
    /// Error message if unsuccessful
    pub error: Option<String>,
}

/// Word with timing information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WordTimestamp {
    pub text: String,
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub speaker_id: Option<String>,
}

// ============================================================================
// Voice Management Tools
// ============================================================================

/// List available voices
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListVoicesInput {
    /// Include detailed information
    #[serde(default)]
    pub detailed: bool,
}

/// Voice information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VoiceInfo {
    pub voice_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub labels: Option<std::collections::HashMap<String, String>>,
}

/// Result of listing voices
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListVoicesOutput {
    pub success: bool,
    pub voices: Vec<VoiceInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

/// Clone a voice from audio samples
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CloneVoiceInput {
    /// Name for the cloned voice
    pub name: String,
    /// Description of the voice
    #[serde(default)]
    pub description: Option<String>,
    /// Paths to audio sample files
    pub samples: Vec<String>,
    /// Labels as key=value pairs
    #[serde(default)]
    pub labels: Option<std::collections::HashMap<String, String>>,
}

/// Result of voice cloning
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CloneVoiceOutput {
    pub success: bool,
    pub voice_id: Option<String>,
    pub requires_verification: Option<bool>,
    pub error: Option<String>,
}

// ============================================================================
// Sound Effects Tools
// ============================================================================

/// Generate sound effects from text
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateSfxInput {
    /// Text description of the sound effect
    pub text: String,
    /// Duration in seconds (0.5-22, auto if not specified)
    #[serde(default)]
    pub duration: Option<f32>,
    /// Prompt influence (0-1)
    #[serde(default)]
    pub influence: Option<f32>,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

/// Result of sound effect generation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateSfxOutput {
    pub success: bool,
    pub output_file: Option<String>,
    pub audio_base64: Option<String>,
    pub duration_seconds: Option<f64>,
    pub error: Option<String>,
}

// ============================================================================
// Audio Isolation Tools
// ============================================================================

/// Isolate vocals/speech from audio
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AudioIsolationInput {
    /// Path to the input audio file
    pub file: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

/// Result of audio isolation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AudioIsolationOutput {
    pub success: bool,
    pub output_file: Option<String>,
    pub audio_base64: Option<String>,
    pub error: Option<String>,
}

// ============================================================================
// Voice Changer Tools
// ============================================================================

/// Transform voice in audio file
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VoiceChangerInput {
    /// Path to the input audio file
    pub file: String,
    /// Target voice ID or name
    pub voice: String,
    /// Model to use
    #[serde(default = "default_vc_model")]
    pub model: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

fn default_vc_model() -> String {
    "eleven_multilingual_sts_v2".to_string()
}

/// Result of voice changing
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VoiceChangerOutput {
    pub success: bool,
    pub output_file: Option<String>,
    pub audio_base64: Option<String>,
    pub error: Option<String>,
}

// ============================================================================
// Dubbing Tools
// ============================================================================

/// Create a dubbing project
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDubbingInput {
    /// Path to the video/audio file
    pub file: String,
    /// Source language code
    pub source_lang: String,
    /// Target language code
    pub target_lang: String,
    /// Number of speakers
    #[serde(default)]
    pub num_speakers: Option<u32>,
}

/// Result of creating dubbing project
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDubbingOutput {
    pub success: bool,
    pub dubbing_id: Option<String>,
    pub expected_duration_sec: Option<f64>,
    pub error: Option<String>,
}

/// Check dubbing status
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetDubbingStatusInput {
    pub dubbing_id: String,
}

/// Dubbing status result
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetDubbingStatusOutput {
    pub success: bool,
    pub status: Option<String>,
    pub name: Option<String>,
    pub target_languages: Option<Vec<String>>,
    pub error: Option<String>,
}

// ============================================================================
// Agent Tools
// ============================================================================

/// List agents
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListAgentsInput {
    /// Maximum number of agents to return
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Agent information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AgentInfo {
    pub agent_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<String>,
}

/// Result of listing agents
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListAgentsOutput {
    pub success: bool,
    pub agents: Vec<AgentInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

/// Create an agent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateAgentInput {
    /// Agent name
    pub name: String,
    /// Agent description
    #[serde(default)]
    pub description: Option<String>,
    /// Voice ID for the agent
    #[serde(default)]
    pub voice_id: Option<String>,
    /// First message template
    #[serde(default)]
    pub first_message: Option<String>,
    /// System prompt
    #[serde(default)]
    pub system_prompt: Option<String>,
}

/// Result of creating agent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateAgentOutput {
    pub success: bool,
    pub agent_id: Option<String>,
    pub error: Option<String>,
}

// ============================================================================
// History Tools
// ============================================================================

/// List generation history
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListHistoryInput {
    /// Maximum number of items to return
    #[serde(default = "default_history_limit")]
    pub limit: u32,
}

fn default_history_limit() -> u32 {
    10
}

/// History item information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HistoryItemInfo {
    pub history_item_id: String,
    pub voice_name: String,
    pub voice_id: String,
    pub model_id: Option<String>,
    pub text: String,
    pub date_unix: i64,
    pub character_count: i32,
}

/// Result of listing history
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListHistoryOutput {
    pub success: bool,
    pub items: Vec<HistoryItemInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

// ============================================================================
// Usage Tools
// ============================================================================

/// Get usage statistics
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetUsageInput {
    /// Start time (Unix timestamp, defaults to 30 days ago)
    #[serde(default)]
    pub start: Option<u64>,
    /// End time (Unix timestamp, defaults to now)
    #[serde(default)]
    pub end: Option<u64>,
}

/// Usage statistics result
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetUsageOutput {
    pub success: bool,
    pub total_characters: u64,
    pub usage_by_type: Option<std::collections::HashMap<String, u64>>,
    pub error: Option<String>,
}

// ============================================================================
// User Tools
// ============================================================================

/// Get user information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetUserInfoInput;

/// User information result
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetUserInfoOutput {
    pub success: bool,
    pub user_id: Option<String>,
    pub subscription_tier: Option<String>,
    pub character_count: Option<i32>,
    pub character_limit: Option<i32>,
    pub usage_percentage: Option<f64>,
    pub error: Option<String>,
}

// ============================================================================
// Model Tools
// ============================================================================

/// List available models
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListModelsInput;

/// Model information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ModelInfo {
    pub model_id: String,
    pub name: String,
    pub description: Option<String>,
    pub languages: Vec<String>,
}

/// Result of listing models
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListModelsOutput {
    pub success: bool,
    pub models: Vec<ModelInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

// ============================================================================
// Dialogue Tools
// ============================================================================

/// Create multi-voice dialogue
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDialogueInput {
    /// Dialogue inputs as array of {text, voice_id} objects
    pub inputs: Vec<DialogueInputItem>,
    /// Model to use
    #[serde(default = "default_dialogue_model")]
    pub model: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

/// Single dialogue input
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DialogueInputItem {
    pub text: String,
    pub voice_id: String,
}

fn default_dialogue_model() -> String {
    "eleven_v3".to_string()
}

/// Result of dialogue creation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDialogueOutput {
    pub success: bool,
    pub output_file: Option<String>,
    pub audio_base64: Option<String>,
    pub voice_segments: Option<Vec<VoiceSegmentInfo>>,
    pub error: Option<String>,
}

/// Voice segment information
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VoiceSegmentInfo {
    pub voice_id: String,
    pub start_time_seconds: f64,
    pub end_time_seconds: f64,
}

// ============================================================================
// Knowledge Base Tools
// ============================================================================

/// Add document to knowledge base
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddKnowledgeInput {
    /// Source type: url, text, or file
    pub source_type: String,
    /// Content (URL, text, or file path)
    pub content: String,
    /// Document name
    pub name: String,
    /// Document description
    #[serde(default)]
    pub description: Option<String>,
}

/// Result of adding knowledge
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddKnowledgeOutput {
    pub success: bool,
    pub document_id: Option<String>,
    pub error: Option<String>,
}

/// List knowledge base documents
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListKnowledgeInput {
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Knowledge document info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDocumentInfo {
    pub id: String,
    pub name: String,
    pub document_type: Option<String>,
    pub created_at: String,
}

/// Result of listing knowledge
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListKnowledgeOutput {
    pub success: bool,
    pub documents: Vec<KnowledgeDocumentInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

// ============================================================================
// Webhook Tools
// ============================================================================

/// Create a webhook
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateWebhookInput {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
}

/// Result of creating webhook
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateWebhookOutput {
    pub success: bool,
    pub webhook_id: Option<String>,
    pub error: Option<String>,
}

/// List webhooks
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListWebhooksInput;

/// Webhook info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WebhookInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
}

/// Result of listing webhooks
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListWebhooksOutput {
    pub success: bool,
    pub webhooks: Vec<WebhookInfo>,
    pub error: Option<String>,
}

// ============================================================================
// Voice Library Tools
// ============================================================================

/// List voices from the shared voice library
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListVoicesFromLibraryInput {
    /// Maximum number of voices to return
    #[serde(default)]
    pub limit: Option<u32>,
    /// Voice category filter (professional, high_quality, generated, famous)
    #[serde(default)]
    pub category: Option<String>,
    /// Gender filter (male, female)
    #[serde(default)]
    pub gender: Option<String>,
    /// Age filter (young, middle_aged, old)
    #[serde(default)]
    pub age: Option<String>,
    /// Search query
    #[serde(default)]
    pub search: Option<String>,
}

/// Result of listing voices from library
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListVoicesFromLibraryOutput {
    pub success: bool,
    pub voices: Vec<VoiceInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

/// Add a voice from the shared library to user's saved voices
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddVoiceToLibraryInput {
    /// Public user ID of the voice owner
    pub public_user_id: String,
    /// Voice ID to add
    pub voice_id: String,
    /// Name for the saved voice
    pub name: String,
}

/// Result of adding voice to library
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddVoiceToLibraryOutput {
    pub success: bool,
    pub voice_id: Option<String>,
    pub error: Option<String>,
}

/// List voice collections
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListCollectionsInput {
    /// Maximum number of collections to return
    #[serde(default)]
    pub page_size: Option<u32>,
}

/// Collection info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CollectionInfo {
    pub collection_id: String,
    pub name: String,
    pub voice_count: Option<u32>,
    pub created_at: Option<String>,
}

/// Result of listing collections
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListCollectionsOutput {
    pub success: bool,
    pub collections: Vec<CollectionInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

/// List voices in a collection
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CollectionVoicesInput {
    /// Collection ID
    pub collection_id: String,
}

/// Collection voice info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CollectionVoiceInfo {
    pub voice_id: String,
    pub name: String,
    pub category: Option<String>,
}

/// Result of listing collection voices
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CollectionVoicesOutput {
    pub success: bool,
    pub voices: Vec<CollectionVoiceInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

// ============================================================================
// Samples Tools
// ============================================================================

/// Get sample audio for a voice
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetSampleAudioInput {
    /// Voice ID
    pub voice_id: String,
    /// Sample ID
    pub sample_id: String,
}

/// Result of getting sample audio
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetSampleAudioOutput {
    pub success: bool,
    pub audio_base64: Option<String>,
    pub error: Option<String>,
}

// ============================================================================
// Pronunciation Tools
// ============================================================================

/// List pronunciation dictionaries
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListDictionariesInput;

/// Pronunciation dictionary info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DictionaryInfo {
    pub id: String,
    pub name: String,
    pub latest_version_id: String,
    pub creation_time_unix: i64,
}

/// Result of listing dictionaries
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListDictionariesOutput {
    pub success: bool,
    pub dictionaries: Vec<DictionaryInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

/// Add a pronunciation dictionary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddDictionaryInput {
    /// Path to the PLS file
    pub file: String,
    /// Dictionary name
    pub name: String,
    /// Dictionary description
    #[serde(default)]
    pub description: Option<String>,
}

/// Result of adding dictionary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddDictionaryOutput {
    pub success: bool,
    pub dictionary_id: Option<String>,
    pub version_id: Option<String>,
    pub error: Option<String>,
}

/// Get a pronunciation dictionary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetDictionaryInput {
    /// Dictionary ID
    pub dictionary_id: String,
}

/// Dictionary detail info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DictionaryDetailInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub latest_version_id: String,
    pub creation_time_unix: i64,
}

/// Result of getting dictionary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetDictionaryOutput {
    pub success: bool,
    pub dictionary: Option<DictionaryDetailInfo>,
    pub error: Option<String>,
}

/// Delete a pronunciation dictionary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteDictionaryInput {
    /// Dictionary ID
    pub dictionary_id: String,
}

/// Result of deleting dictionary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteDictionaryOutput {
    pub success: bool,
    pub error: Option<String>,
}

/// List rules in a pronunciation dictionary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListRulesInput {
    /// Dictionary ID
    pub dictionary_id: String,
}

/// Pronunciation rule
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PronunciationRule {
    pub word: String,
    pub phoneme: Option<String>,
    pub aliases: Option<Vec<String>>,
}

/// Result of listing rules
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListRulesOutput {
    pub success: bool,
    pub rules: Vec<PronunciationRule>,
    pub total_count: usize,
    pub error: Option<String>,
}

/// Add rules to a pronunciation dictionary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddRulesInput {
    /// Dictionary ID
    pub dictionary_id: String,
    /// Rules file path (JSON format)
    pub rules_file: String,
}

/// Result of adding rules
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddRulesOutput {
    pub success: bool,
    pub error: Option<String>,
}

/// Remove rules from a pronunciation dictionary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RemoveRulesInput {
    /// Dictionary ID
    pub dictionary_id: String,
    /// Rules file path (JSON format)
    pub rules_file: String,
}

/// Result of removing rules
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RemoveRulesOutput {
    pub success: bool,
    pub error: Option<String>,
}

// ============================================================================
// History Tools (Additional)
// ============================================================================

/// Get a specific history item
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetHistoryItemInput {
    /// History item ID
    pub history_item_id: String,
}

/// Result of getting history item
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetHistoryItemOutput {
    pub success: bool,
    pub history_item: Option<HistoryItemInfo>,
    pub error: Option<String>,
}

/// Delete a history item
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteHistoryItemInput {
    /// History item ID
    pub history_item_id: String,
}

/// Result of deleting history item
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteHistoryItemOutput {
    pub success: bool,
    pub error: Option<String>,
}

/// Submit feedback for a history item
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SubmitFeedbackInput {
    /// History item ID
    pub history_item_id: String,
    /// Thumbs up or down
    pub thumbs_up: bool,
    /// Optional feedback text
    #[serde(default)]
    pub feedback: Option<String>,
}

/// Result of submitting feedback
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SubmitFeedbackOutput {
    pub success: bool,
    pub error: Option<String>,
}

// ============================================================================
// RAG Tools
// ============================================================================

/// Rebuild RAG index for a document
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RebuildIndexInput {
    /// Document ID
    pub document_id: String,
}

/// Result of rebuilding index
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RebuildIndexOutput {
    pub success: bool,
    pub index_id: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

/// Get RAG index status for a document
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetIndexStatusInput {
    /// Document ID
    pub document_id: String,
}

/// Index status info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexStatusInfo {
    pub id: Option<String>,
    pub status: Option<String>,
    pub active: Option<bool>,
    pub document_id: Option<String>,
    pub error: Option<String>,
}

/// Result of getting index status
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetIndexStatusOutput {
    pub success: bool,
    pub status: Option<IndexStatusInfo>,
    pub error: Option<String>,
}

// ============================================================================
// Audio Native Tools
// ============================================================================

/// List Audio Native projects
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListAudioNativeInput {
    /// Maximum number of projects to return
    #[serde(default = "default_limit_10")]
    pub limit: u32,
    /// Page number
    #[serde(default = "default_page_1")]
    pub page: u32,
}

fn default_limit_10() -> u32 {
    10
}

fn default_page_1() -> u32 {
    1
}

/// Audio Native project info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AudioNativeProjectInfo {
    pub project_id: String,
    pub name: String,
    pub author: Option<String>,
    pub title: Option<String>,
    pub voice_id: Option<String>,
    pub model_id: Option<String>,
    pub created_at: String,
}

/// Result of listing Audio Native projects
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListAudioNativeOutput {
    pub success: bool,
    pub projects: Vec<AudioNativeProjectInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

/// Get an Audio Native project
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetAudioNativeInput {
    /// Project ID
    pub project_id: String,
}

/// Result of getting Audio Native project
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetAudioNativeOutput {
    pub success: bool,
    pub project: Option<AudioNativeProjectInfo>,
    pub error: Option<String>,
}

// ============================================================================
// Conversation Tools
// ============================================================================

/// Get a conversation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetConversationInput {
    /// Conversation ID
    pub conversation_id: String,
}

/// Transcript message
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TranscriptMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

/// Conversation detail
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ConversationDetailInfo {
    pub conversation_id: String,
    pub agent_id: Option<String>,
    pub version_id: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
    pub transcript: Option<Vec<TranscriptMessage>>,
}

/// Result of getting conversation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetConversationOutput {
    pub success: bool,
    pub conversation: Option<ConversationDetailInfo>,
    pub error: Option<String>,
}

/// Delete a conversation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteConversationInput {
    /// Conversation ID
    pub conversation_id: String,
}

/// Result of deleting conversation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteConversationOutput {
    pub success: bool,
    pub error: Option<String>,
}

/// Get conversation audio
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetConversationAudioInput {
    /// Conversation ID
    pub conversation_id: String,
    /// Output file path
    #[serde(default)]
    pub output: Option<String>,
}

/// Result of getting conversation audio
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetConversationAudioOutput {
    pub success: bool,
    pub output_file: Option<String>,
    pub audio_base64: Option<String>,
    pub error: Option<String>,
}

// ============================================================================
// Workspace Tools
// ============================================================================

/// List workspace members
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListMembersInput;

/// Workspace member info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MemberInfo {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub joined_at: Option<String>,
}

/// Result of listing members
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListMembersOutput {
    pub success: bool,
    pub members: Vec<MemberInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

/// Invite a member to the workspace
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InviteMemberInput {
    /// Email address
    pub email: String,
    /// Role (admin, editor, viewer, contributor)
    pub role: String,
}

/// Result of inviting member
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InviteMemberOutput {
    pub success: bool,
    pub error: Option<String>,
}

/// Revoke an invitation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RevokeInviteInput {
    /// Email address
    pub email: String,
}

/// Result of revoking invite
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RevokeInviteOutput {
    pub success: bool,
    pub error: Option<String>,
}

/// List workspace secrets
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListSecretsInput;

/// Secret info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SecretInfo {
    pub name: String,
    pub secret_type: String,
    pub created_at: Option<String>,
}

/// Result of listing secrets
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListSecretsOutput {
    pub success: bool,
    pub secrets: Vec<SecretInfo>,
    pub total_count: usize,
    pub error: Option<String>,
}

/// Add a workspace secret
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddSecretInput {
    /// Secret name
    pub name: String,
    /// Secret value
    pub value: String,
    /// Secret type (api_key, env_variable, secret)
    pub secret_type: String,
}

/// Result of adding secret
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddSecretOutput {
    pub success: bool,
    pub error: Option<String>,
}

/// Delete a workspace secret
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteSecretInput {
    /// Secret name
    pub name: String,
}

/// Result of deleting secret
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteSecretOutput {
    pub success: bool,
    pub error: Option<String>,
}

// ============================================================================
// Phone Tools
// ============================================================================

/// Get a phone number
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetPhoneNumberInput {
    /// Phone number ID
    pub phone_id: String,
}

/// Phone number info
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PhoneNumberInfo {
    pub phone_number_id: String,
    pub phone_number: String,
    pub label: Option<String>,
    pub provider: Option<String>,
    pub agent_id: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
}

/// Result of getting phone number
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetPhoneNumberOutput {
    pub success: bool,
    pub phone_number: Option<PhoneNumberInfo>,
    pub error: Option<String>,
}

/// Delete a phone number
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeletePhoneNumberInput {
    /// Phone number ID
    pub phone_id: String,
}

/// Result of deleting phone number
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeletePhoneNumberOutput {
    pub success: bool,
    pub error: Option<String>,
}
