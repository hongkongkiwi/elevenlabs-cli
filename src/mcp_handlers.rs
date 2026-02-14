//! MCP tool handlers - bridges between MCP tools and CLI commands.
//!
//! This module implements the actual logic for MCP tools by reusing
//! the existing command implementations.

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

#[cfg(feature = "mcp")]
use crate::mcp::tools::*;

#[cfg(feature = "mcp")]
use crate::utils::parse_output_format;

// ============================================================================
// TTS Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn text_to_speech(api_key: &str, input: TextToSpeechInput) -> Result<TextToSpeechOutput> {
    use elevenlabs_rs::{
        endpoints::genai::tts::{TextToSpeech, TextToSpeechBody, TextToSpeechQuery},
        ElevenLabsClient, Model, VoiceSettings,
    };

    let client = ElevenLabsClient::new(api_key);

    // Parse model
    let model = match input.model.as_str() {
        "eleven_multilingual_v2" => Model::ElevenMultilingualV2,
        "eleven_flash_v2_5" => Model::ElevenFlashV2_5,
        "eleven_turbo_v2" => Model::ElevenTurboV2,
        "eleven_turbo_v2_5" => Model::ElevenTurboV2_5,
        "eleven_v3" => Model::ElevenMultilingualV2,
        _ => Model::ElevenMultilingualV2,
    };

    // Parse output format using shared utility
    let format = parse_output_format(&input.output_format)?;

    // Build voice settings
    let voice_settings = if input.stability.is_some()
        || input.similarity_boost.is_some()
        || input.style.is_some()
        || input.speaker_boost
    {
        let mut settings = VoiceSettings::default();
        if let Some(s) = input.stability {
            settings = settings.with_stability(s);
        }
        if let Some(sb) = input.similarity_boost {
            settings = settings.with_similarity_boost(sb);
        }
        if let Some(st) = input.style {
            settings = settings.with_style(st);
        }
        settings = settings.use_speaker_boost(input.speaker_boost);
        Some(settings)
    } else {
        None
    };

    // Build request body
    let mut body = TextToSpeechBody::new(&input.text).with_model_id(model);

    if let Some(settings) = voice_settings {
        body = body.with_voice_settings(settings);
    }

    let query = TextToSpeechQuery::default().with_output_format(format);

    let endpoint = TextToSpeech::new(&input.voice, body).with_query(query);

    let start = std::time::Instant::now();
    let audio = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    let duration = start.elapsed().as_secs_f64();

    // Handle output
    if let Some(output_file) = input.output_file {
        std::fs::write(&output_file, &audio)?;
        Ok(TextToSpeechOutput {
            success: true,
            output_file: Some(output_file),
            audio_base64: None,
            duration_seconds: Some(duration),
            error: None,
        })
    } else {
        let audio_base64 = BASE64.encode(&audio);
        Ok(TextToSpeechOutput {
            success: true,
            output_file: None,
            audio_base64: Some(audio_base64),
            duration_seconds: Some(duration),
            error: None,
        })
    }
}

// ============================================================================
// STT Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn speech_to_text(api_key: &str, input: SpeechToTextInput) -> Result<SpeechToTextOutput> {
    use elevenlabs_rs::{
        endpoints::genai::speech_to_text::{
            CreateTranscript, CreateTranscriptBody, Granularity, SpeechToTextModel,
        },
        ElevenLabsClient,
    };

    let client = ElevenLabsClient::new(api_key);

    let model = match input.model.as_str() {
        "scribe_v1" => SpeechToTextModel::ScribeV1,
        "scribe_v1_base" => SpeechToTextModel::ScribeV1Base,
        "scribe_v2" => SpeechToTextModel::ScribeV1, // Fallback
        _ => SpeechToTextModel::ScribeV1,
    };

    let mut body = CreateTranscriptBody::new(model, &input.file).with_tag_audio_events(true);

    if let Some(lang) = input.language {
        body = body.with_language_code(&lang);
    }

    if let Some(speakers) = input.num_speakers {
        body = body.with_num_speakers(speakers);
    }

    let timestamps = match input.timestamps.as_str() {
        "none" => Granularity::None,
        "character" => Granularity::Character,
        _ => Granularity::Word,
    };
    body = body.with_timestamps_granularity(timestamps);

    if input.diarize {
        body = body.with_diarize(true);
    }

    let endpoint = CreateTranscript::new(body);
    let result = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let words: Vec<WordTimestamp> = result
        .words
        .iter()
        .map(|w| WordTimestamp {
            text: w.text.clone(),
            start: w.start.map(|f| f as f64),
            end: w.end.map(|f| f as f64),
            speaker_id: w.speaker_id.clone(),
        })
        .collect();

    let full_text: String = result
        .words
        .iter()
        .map(|w| w.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(SpeechToTextOutput {
        success: true,
        text: Some(full_text),
        language_code: Some(result.language_code.clone()),
        language_probability: Some(result.language_probability as f64),
        words: Some(words),
        error: None,
    })
}

// ============================================================================
// Voice Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn list_voices(api_key: &str, _input: ListVoicesInput) -> Result<ListVoicesOutput> {
    use elevenlabs_rs::{endpoints::admin::voice::GetVoices, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);
    let endpoint = GetVoices::default();
    let voices = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let voice_infos: Vec<VoiceInfo> = voices
        .voices
        .iter()
        .map(|v| VoiceInfo {
            voice_id: v.voice_id.clone(),
            name: v.name.clone().unwrap_or_else(|| "Unknown".to_string()),
            category: v.category.as_ref().map(|c| format!("{:?}", c)),
            description: v.description.clone(),
            labels: v.labels.clone(),
        })
        .collect();

    let total = voice_infos.len();

    Ok(ListVoicesOutput {
        success: true,
        voices: voice_infos,
        total_count: total,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn clone_voice(api_key: &str, input: CloneVoiceInput) -> Result<CloneVoiceOutput> {
    use elevenlabs_rs::{
        endpoints::admin::voice::{AddVoice, VoiceBody},
        ElevenLabsClient,
    };

    let client = ElevenLabsClient::new(api_key);

    let mut body = VoiceBody::add(&input.name, input.samples.clone());

    if let Some(desc) = input.description {
        body = body.with_description(&desc);
    }

    if let Some(labels) = input.labels {
        let labels_vec: Vec<(String, String)> = labels.into_iter().collect();
        if !labels_vec.is_empty() {
            body = body.with_labels(labels_vec);
        }
    }

    let endpoint = AddVoice::new(body);
    let response = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(CloneVoiceOutput {
        success: true,
        voice_id: Some(response.voice_id),
        requires_verification: Some(response.requires_verification),
        error: None,
    })
}

// ============================================================================
// Sound Effects Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn generate_sfx(api_key: &str, input: GenerateSfxInput) -> Result<GenerateSfxOutput> {
    use elevenlabs_rs::{
        endpoints::genai::sound_effects::{CreateSoundEffect, CreateSoundEffectBody},
        ElevenLabsClient,
    };

    let client = ElevenLabsClient::new(api_key);

    let mut body = CreateSoundEffectBody::new(&input.text);

    if let Some(duration) = input.duration {
        body = body.with_duration_seconds(duration);
    }

    if let Some(influence) = input.influence {
        body = body.with_prompt_influence(influence);
    }

    let endpoint = CreateSoundEffect::new(body);
    let audio = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Some(output_file) = input.output_file {
        std::fs::write(&output_file, &audio)?;
        Ok(GenerateSfxOutput {
            success: true,
            output_file: Some(output_file),
            audio_base64: None,
            duration_seconds: None,
            error: None,
        })
    } else {
        let audio_base64 = BASE64.encode(&audio);
        Ok(GenerateSfxOutput {
            success: true,
            output_file: None,
            audio_base64: Some(audio_base64),
            duration_seconds: None,
            error: None,
        })
    }
}

// ============================================================================
// Audio Isolation Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn audio_isolation(
    api_key: &str,
    input: AudioIsolationInput,
) -> Result<AudioIsolationOutput> {
    use elevenlabs_rs::{endpoints::genai::audio_isolation::AudioIsolation, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);
    let endpoint = AudioIsolation::new(input.file.clone());
    let audio = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Some(output_file) = input.output_file {
        std::fs::write(&output_file, &audio)?;
        Ok(AudioIsolationOutput {
            success: true,
            output_file: Some(output_file),
            audio_base64: None,
            error: None,
        })
    } else {
        let audio_base64 = BASE64.encode(&audio);
        Ok(AudioIsolationOutput {
            success: true,
            output_file: None,
            audio_base64: Some(audio_base64),
            error: None,
        })
    }
}

// ============================================================================
// Voice Changer Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn voice_changer(api_key: &str, input: VoiceChangerInput) -> Result<VoiceChangerOutput> {
    use elevenlabs_rs::{
        endpoints::genai::voice_changer::{
            VoiceChanger as VcEndpoint, VoiceChangerBody, VoiceChangerQuery,
        },
        ElevenLabsClient, Model, OutputFormat,
    };

    let client = ElevenLabsClient::new(api_key);

    let model = match input.model.as_str() {
        "eleven_english_sts_v2" => Model::ElevenEnglishV2,
        _ => Model::ElevenMultilingualV2STS,
    };

    let body = VoiceChangerBody::new(input.file.clone()).with_model_id(model);

    let query = VoiceChangerQuery::default().with_output_format(OutputFormat::Mp3_44100Hz128kbps);

    let endpoint = VcEndpoint::new(&input.voice, body).with_query(query);

    let audio = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Some(output_file) = input.output_file {
        std::fs::write(&output_file, &audio)?;
        Ok(VoiceChangerOutput {
            success: true,
            output_file: Some(output_file),
            audio_base64: None,
            error: None,
        })
    } else {
        let audio_base64 = BASE64.encode(&audio);
        Ok(VoiceChangerOutput {
            success: true,
            output_file: None,
            audio_base64: Some(audio_base64),
            error: None,
        })
    }
}

// ============================================================================
// Dubbing Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn create_dubbing(
    api_key: &str,
    input: CreateDubbingInput,
) -> Result<CreateDubbingOutput> {
    use elevenlabs_rs::{
        endpoints::genai::dubbing::{DubAVideoOrAnAudioFile, DubbingBody},
        ElevenLabsClient,
    };

    let client = ElevenLabsClient::new(api_key);

    let mut body = DubbingBody::new(&input.target_lang)
        .with_file(&input.file)
        .with_source_lang(&input.source_lang);

    if let Some(speakers) = input.num_speakers {
        body = body.with_num_speakers(speakers);
    }

    let endpoint = DubAVideoOrAnAudioFile::new(body);
    let response = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(CreateDubbingOutput {
        success: true,
        dubbing_id: Some(response.dubbing_id),
        expected_duration_sec: Some(response.expected_duration_sec as f64),
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn get_dubbing_status(
    api_key: &str,
    input: GetDubbingStatusInput,
) -> Result<GetDubbingStatusOutput> {
    use elevenlabs_rs::{endpoints::genai::dubbing::GetDubbing, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);
    let endpoint = GetDubbing::new(&input.dubbing_id);
    let status = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(GetDubbingStatusOutput {
        success: true,
        status: Some(status.status.clone()),
        name: Some(status.name),
        target_languages: Some(status.target_languages),
        error: None,
    })
}

// ============================================================================
// Agent Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn list_agents(api_key: &str, input: ListAgentsInput) -> Result<ListAgentsOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct AgentSummary {
        agent_id: String,
        name: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        created_at: Option<String>,
    }

    let client = Client::new();
    let mut url = "https://api.elevenlabs.io/v1/agents".to_string();

    if let Some(limit) = input.limit {
        url.push_str(&format!("?limit={}", limit));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(ListAgentsOutput {
            success: false,
            agents: vec![],
            total_count: 0,
            error: Some(error),
        });
    }

    let agents: Vec<AgentSummary> = response.json().await?;

    let agent_infos: Vec<AgentInfo> = agents
        .iter()
        .map(|a| AgentInfo {
            agent_id: a.agent_id.clone(),
            name: a.name.clone(),
            description: a.description.clone(),
            created_at: a.created_at.clone(),
        })
        .collect();

    let total = agent_infos.len();

    Ok(ListAgentsOutput {
        success: true,
        agents: agent_infos,
        total_count: total,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn create_agent(api_key: &str, input: CreateAgentInput) -> Result<CreateAgentOutput> {
    use reqwest::Client;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    struct CreateResponse {
        agent_id: String,
    }

    let client = Client::new();

    let mut body = json!({
        "name": input.name
    });

    if let Some(desc) = input.description {
        body["description"] = json!(desc);
    }

    // Add conversation config
    let mut conversation_config = json!({});
    if let Some(voice_id) = input.voice_id {
        conversation_config["agent"]["voice_id"] = json!(voice_id);
    }
    if let Some(first_message) = input.first_message {
        conversation_config["agent"]["first_message"] = json!(first_message);
    }
    if let Some(system_prompt) = input.system_prompt {
        conversation_config["agent"]["prompt"]["prompt"] = json!(system_prompt);
    }
    body["conversation_config"] = conversation_config;

    let response = client
        .post("https://api.elevenlabs.io/v1/agents")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(CreateAgentOutput {
            success: false,
            agent_id: None,
            error: Some(error),
        });
    }

    let result: CreateResponse = response.json().await?;

    Ok(CreateAgentOutput {
        success: true,
        agent_id: Some(result.agent_id),
        error: None,
    })
}

// ============================================================================
// History Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn list_history(api_key: &str, input: ListHistoryInput) -> Result<ListHistoryOutput> {
    use elevenlabs_rs::{
        endpoints::admin::history::{GetGeneratedItems, HistoryQuery},
        ElevenLabsClient,
    };

    let client = ElevenLabsClient::new(api_key);

    let query = HistoryQuery::default().with_page_size(input.limit as u16);

    let endpoint = GetGeneratedItems::with_query(query);
    let history = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let items: Vec<HistoryItemInfo> = history
        .history
        .iter()
        .map(|h| HistoryItemInfo {
            history_item_id: h.history_item_id.clone(),
            voice_name: h.voice_name.clone(),
            voice_id: h.voice_id.clone(),
            model_id: h.model_id.clone(),
            text: h.text.chars().take(200).collect(),
            date_unix: h.date_unix as i64,
            character_count: h.character_count_change_from as i32,
        })
        .collect();

    let total = items.len();

    Ok(ListHistoryOutput {
        success: true,
        items,
        total_count: total,
        error: None,
    })
}

// ============================================================================
// Usage Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn get_usage(api_key: &str, input: GetUsageInput) -> Result<GetUsageOutput> {
    use elevenlabs_rs::{
        endpoints::admin::usage::{GetUsage, GetUsageQuery},
        ElevenLabsClient,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    let client = ElevenLabsClient::new(api_key);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let start = input.start.unwrap_or(now - 30 * 24 * 60 * 60);
    let end = input.end.unwrap_or(now);

    let query = GetUsageQuery::new(start, end);
    let endpoint = GetUsage::new(query);
    let response = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let total: u64 = response.usage.values().flatten().sum();
    let usage_by_type: std::collections::HashMap<String, u64> = response
        .usage
        .iter()
        .map(|(k, v)| {
            let sum: u64 = v.iter().sum();
            (k.clone(), sum)
        })
        .collect();

    Ok(GetUsageOutput {
        success: true,
        total_characters: total,
        usage_by_type: Some(usage_by_type),
        error: None,
    })
}

// ============================================================================
// User Info Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn get_user_info(api_key: &str) -> Result<GetUserInfoOutput> {
    use elevenlabs_rs::{endpoints::admin::user::GetUserInfo, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);
    let endpoint = GetUserInfo;
    let user = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let usage_percentage = if user.subscription.character_limit > 0 {
        Some(
            (user.subscription.character_count as f64 / user.subscription.character_limit as f64)
                * 100.0,
        )
    } else {
        None
    };

    Ok(GetUserInfoOutput {
        success: true,
        user_id: Some(user.user_id),
        subscription_tier: Some(user.subscription.tier),
        character_count: Some(user.subscription.character_count as i32),
        character_limit: Some(user.subscription.character_limit as i32),
        usage_percentage,
        error: None,
    })
}

// ============================================================================
// Models Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn list_models(api_key: &str) -> Result<ListModelsOutput> {
    use elevenlabs_rs::{endpoints::admin::models::GetModels, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);
    let endpoint = GetModels;
    let models = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let model_infos: Vec<ModelInfo> = models
        .iter()
        .map(|m| ModelInfo {
            model_id: m.model_id.clone(),
            name: m.name.clone(),
            description: Some(m.description.clone()),
            languages: m.languages.iter().map(|l| l.language_id.clone()).collect(),
        })
        .collect();

    let total = model_infos.len();

    Ok(ListModelsOutput {
        success: true,
        models: model_infos,
        total_count: total,
        error: None,
    })
}

// ============================================================================
// Dialogue Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn create_dialogue(
    api_key: &str,
    input: CreateDialogueInput,
) -> Result<CreateDialogueOutput> {
    use reqwest::Client;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    struct DialogueResponse {
        audio_base64: String,
        voice_segments: Option<Vec<VoiceSegmentRaw>>,
    }

    #[derive(Deserialize)]
    struct VoiceSegmentRaw {
        voice_id: String,
        start_time_seconds: f64,
        end_time_seconds: f64,
    }

    let client = Client::new();

    let dialogue_inputs: Vec<_> = input
        .inputs
        .iter()
        .map(|i| json!({ "text": i.text, "voice_id": i.voice_id }))
        .collect();

    let body = json!({
        "inputs": dialogue_inputs,
        "model_id": input.model,
        "output_format": "mp3_44100_128"
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/text-to-dialogue/stream/with-timestamps")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(CreateDialogueOutput {
            success: false,
            output_file: None,
            audio_base64: None,
            voice_segments: None,
            error: Some(error),
        });
    }

    let result: DialogueResponse = response.json().await?;

    let voice_segments = result.voice_segments.map(|segments| {
        segments
            .iter()
            .map(|s| VoiceSegmentInfo {
                voice_id: s.voice_id.clone(),
                start_time_seconds: s.start_time_seconds,
                end_time_seconds: s.end_time_seconds,
            })
            .collect()
    });

    if let Some(output_file) = input.output_file {
        let audio = BASE64.decode(&result.audio_base64)?;
        std::fs::write(&output_file, &audio)?;
        Ok(CreateDialogueOutput {
            success: true,
            output_file: Some(output_file),
            audio_base64: None,
            voice_segments,
            error: None,
        })
    } else {
        Ok(CreateDialogueOutput {
            success: true,
            output_file: None,
            audio_base64: Some(result.audio_base64),
            voice_segments,
            error: None,
        })
    }
}

// ============================================================================
// Knowledge Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn add_knowledge(api_key: &str, input: AddKnowledgeInput) -> Result<AddKnowledgeOutput> {
    use reqwest::Client;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    struct CreateResponse {
        id: String,
    }

    let client = Client::new();

    let body = match input.source_type.as_str() {
        "url" => json!({
            "name": input.name,
            "type": "url",
            "url": input.content,
            "description": input.description.unwrap_or_default()
        }),
        "file" => {
            let content = std::fs::read_to_string(&input.content)?;
            json!({
                "name": input.name,
                "type": "text",
                "content": content
            })
        }
        _ => json!({
            "name": input.name,
            "type": "text",
            "content": input.content,
            "description": input.description.unwrap_or_default()
        }),
    };

    let response = client
        .post("https://api.elevenlabs.io/v1/convai/knowledge-base")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(AddKnowledgeOutput {
            success: false,
            document_id: None,
            error: Some(error),
        });
    }

    let result: CreateResponse = response.json().await?;

    Ok(AddKnowledgeOutput {
        success: true,
        document_id: Some(result.id),
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn list_knowledge(
    api_key: &str,
    input: ListKnowledgeInput,
) -> Result<ListKnowledgeOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct KnowledgeDocument {
        id: String,
        name: String,
        #[serde(default)]
        document_type: Option<String>,
        created_at: String,
    }

    let client = Client::new();
    let mut url = "https://api.elevenlabs.io/v1/convai/knowledge-base".to_string();

    if let Some(limit) = input.limit {
        url.push_str(&format!("?limit={}", limit));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(ListKnowledgeOutput {
            success: false,
            documents: vec![],
            total_count: 0,
            error: Some(error),
        });
    }

    let documents: Vec<KnowledgeDocument> = response.json().await?;

    let doc_infos: Vec<KnowledgeDocumentInfo> = documents
        .iter()
        .map(|d| KnowledgeDocumentInfo {
            id: d.id.clone(),
            name: d.name.clone(),
            document_type: d.document_type.clone(),
            created_at: d.created_at.clone(),
        })
        .collect();

    let total = doc_infos.len();

    Ok(ListKnowledgeOutput {
        success: true,
        documents: doc_infos,
        total_count: total,
        error: None,
    })
}

// ============================================================================
// Webhook Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn create_webhook(
    api_key: &str,
    input: CreateWebhookInput,
) -> Result<CreateWebhookOutput> {
    use reqwest::Client;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize)]
    struct CreateResponse {
        id: String,
    }

    let client = Client::new();

    let body = json!({
        "name": input.name,
        "url": input.url,
        "events": input.events
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/webhooks")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(CreateWebhookOutput {
            success: false,
            webhook_id: None,
            error: Some(error),
        });
    }

    let result: CreateResponse = response.json().await?;

    Ok(CreateWebhookOutput {
        success: true,
        webhook_id: Some(result.id),
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn list_webhooks(api_key: &str) -> Result<ListWebhooksOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct WebhookData {
        id: String,
        name: String,
        url: String,
        events: Vec<String>,
    }

    let client = Client::new();

    let response = client
        .get("https://api.elevenlabs.io/v1/webhooks")
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(ListWebhooksOutput {
            success: false,
            webhooks: vec![],
            error: Some(error),
        });
    }

    let webhooks: Vec<WebhookData> = response.json().await?;

    let webhook_infos: Vec<WebhookInfo> = webhooks
        .iter()
        .map(|w| WebhookInfo {
            id: w.id.clone(),
            name: w.name.clone(),
            url: w.url.clone(),
            events: w.events.clone(),
        })
        .collect();

    Ok(ListWebhooksOutput {
        success: true,
        webhooks: webhook_infos,
        error: None,
    })
}

// ============================================================================
// Voice Library Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn list_voices_from_library(
    api_key: &str,
    input: ListVoicesFromLibraryInput,
) -> Result<ListVoicesFromLibraryOutput> {
    use elevenlabs_rs::{
        endpoints::admin::voice_library::{
            Age, Gender, GetSharedVoices, SharedVoiceCategory, SharedVoicesQuery,
        },
        ElevenLabsClient,
    };

    let client = ElevenLabsClient::new(api_key);

    let mut query = SharedVoicesQuery::default();

    if let Some(ps) = input.limit {
        query = query.with_page_size(ps as u16);
    }

    if let Some(cat) = input.category {
        let cat_enum = match cat.to_lowercase().as_str() {
            "professional" => SharedVoiceCategory::Professional,
            "high_quality" => SharedVoiceCategory::HighQuality,
            "generated" => SharedVoiceCategory::Generated,
            "famous" => SharedVoiceCategory::Famous,
            _ => SharedVoiceCategory::HighQuality,
        };
        query = query.with_category(cat_enum);
    }

    if let Some(g) = input.gender {
        let gender_enum = match g.to_lowercase().as_str() {
            "male" => Gender::Male,
            "female" => Gender::Female,
            _ => Gender::Female,
        };
        query = query.with_gender(gender_enum);
    }

    if let Some(a) = input.age {
        let age_enum = match a.to_lowercase().as_str() {
            "young" => Age::Young,
            "middle_aged" => Age::MiddleAged,
            "old" => Age::Old,
            _ => Age::Young,
        };
        query = query.with_age(age_enum);
    }

    if let Some(s) = input.search {
        query = query.with_search(&s);
    }

    let endpoint = GetSharedVoices::with_query(query);
    let response = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let voice_infos: Vec<VoiceInfo> = response
        .voices
        .iter()
        .map(|v| VoiceInfo {
            voice_id: v.voice_id.clone(),
            name: v.name.clone(),
            category: Some(format!("{:?}", v.category)),
            description: v.description.clone(),
            labels: None,
        })
        .collect();

    let total = voice_infos.len();

    Ok(ListVoicesFromLibraryOutput {
        success: true,
        voices: voice_infos,
        total_count: total,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn add_voice_to_library(
    api_key: &str,
    input: AddVoiceToLibraryInput,
) -> Result<AddVoiceToLibraryOutput> {
    use elevenlabs_rs::{endpoints::admin::voice_library::AddSharedVoice, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);

    let endpoint = AddSharedVoice::new(&input.public_user_id, &input.voice_id, &input.name);
    let response = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(AddVoiceToLibraryOutput {
        success: true,
        voice_id: Some(response.voice_id),
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn list_collections(
    api_key: &str,
    input: ListCollectionsInput,
) -> Result<ListCollectionsOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct CollectionsResponse {
        collections: Vec<CollectionData>,
    }

    #[derive(Deserialize)]
    struct CollectionData {
        collection_id: String,
        name: String,
        #[serde(default)]
        voice_count: Option<u32>,
        #[serde(default)]
        created_at: Option<String>,
    }

    let client = Client::new();
    let mut url = "https://api.elevenlabs.io/v1/voices/collections".to_string();

    if let Some(ps) = input.page_size {
        url.push_str(&format!("?page_size={}", ps));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(ListCollectionsOutput {
            success: false,
            collections: vec![],
            total_count: 0,
            error: Some(error),
        });
    }

    let result: CollectionsResponse = response.json().await?;

    let collection_infos: Vec<CollectionInfo> = result
        .collections
        .iter()
        .map(|c| CollectionInfo {
            collection_id: c.collection_id.clone(),
            name: c.name.clone(),
            voice_count: c.voice_count,
            created_at: c.created_at.clone(),
        })
        .collect();

    let total = collection_infos.len();

    Ok(ListCollectionsOutput {
        success: true,
        collections: collection_infos,
        total_count: total,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn collection_voices(
    api_key: &str,
    input: CollectionVoicesInput,
) -> Result<CollectionVoicesOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct CollectionVoicesResponse {
        voices: Vec<CollectionVoiceData>,
    }

    #[derive(Deserialize)]
    struct CollectionVoiceData {
        voice_id: String,
        name: String,
        #[serde(default)]
        category: Option<String>,
    }

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/voices/collections/{}/voices",
        input.collection_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(CollectionVoicesOutput {
            success: false,
            voices: vec![],
            total_count: 0,
            error: Some(error),
        });
    }

    let result: CollectionVoicesResponse = response.json().await?;

    let voice_infos: Vec<CollectionVoiceInfo> = result
        .voices
        .iter()
        .map(|v| CollectionVoiceInfo {
            voice_id: v.voice_id.clone(),
            name: v.name.clone(),
            category: v.category.clone(),
        })
        .collect();

    let total = voice_infos.len();

    Ok(CollectionVoicesOutput {
        success: true,
        voices: voice_infos,
        total_count: total,
        error: None,
    })
}

// ============================================================================
// Samples Handler
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn get_sample_audio(
    api_key: &str,
    input: GetSampleAudioInput,
) -> Result<GetSampleAudioOutput> {
    use elevenlabs_rs::{endpoints::admin::samples::GetAudioFromSample, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);

    let endpoint = GetAudioFromSample::new(&input.voice_id, &input.sample_id);
    let audio = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let audio_base64 = BASE64.encode(&audio);

    Ok(GetSampleAudioOutput {
        success: true,
        audio_base64: Some(audio_base64),
        error: None,
    })
}

// ============================================================================
// Pronunciation Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn list_dictionaries_ex(
    api_key: &str,
    _input: ListDictionariesInput,
) -> Result<ListDictionariesOutput> {
    use elevenlabs_rs::{endpoints::admin::pronunciation::GetDictionaries, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);

    let endpoint = GetDictionaries::default();
    let response = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let dictionaries: Vec<DictionaryInfo> = response
        .pronunciation_dictionaries
        .iter()
        .map(|d| DictionaryInfo {
            id: d.id.clone(),
            name: d.name.clone(),
            latest_version_id: d.latest_version_id.clone(),
            creation_time_unix: d.creation_time_unix as i64,
        })
        .collect();

    let total = dictionaries.len();

    Ok(ListDictionariesOutput {
        success: true,
        dictionaries,
        total_count: total,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn add_dictionary_ex(
    api_key: &str,
    input: AddDictionaryInput,
) -> Result<AddDictionaryOutput> {
    use elevenlabs_rs::{
        endpoints::admin::pronunciation::{CreateDictionary, CreateDictionaryBody},
        ElevenLabsClient,
    };

    let client = ElevenLabsClient::new(api_key);

    let mut body = CreateDictionaryBody::new(&input.file, &input.name);

    if let Some(desc) = input.description {
        body = body.with_description(&desc);
    }

    let endpoint = CreateDictionary::new(body);
    let response = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(AddDictionaryOutput {
        success: true,
        dictionary_id: Some(response.id),
        version_id: Some(response.version_id),
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn get_dictionary_ex(
    api_key: &str,
    input: GetDictionaryInput,
) -> Result<GetDictionaryOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct DictionaryDetail {
        id: String,
        name: String,
        #[serde(default)]
        description: Option<String>,
        latest_version_id: String,
        creation_time_unix: i64,
    }

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation-dictionaries/{}",
        input.dictionary_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(GetDictionaryOutput {
            success: false,
            dictionary: None,
            error: Some(error),
        });
    }

    let dict: DictionaryDetail = response.json().await?;

    Ok(GetDictionaryOutput {
        success: true,
        dictionary: Some(DictionaryDetailInfo {
            id: dict.id,
            name: dict.name,
            description: dict.description,
            latest_version_id: dict.latest_version_id,
            creation_time_unix: dict.creation_time_unix,
        }),
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn delete_dictionary_ex(
    api_key: &str,
    input: DeleteDictionaryInput,
) -> Result<DeleteDictionaryOutput> {
    use reqwest::Client;

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation-dictionaries/{}",
        input.dictionary_id
    );

    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(DeleteDictionaryOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(DeleteDictionaryOutput {
        success: true,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn list_rules_ex(api_key: &str, input: ListRulesInput) -> Result<ListRulesOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct RulesResponse {
        rules: Vec<RuleData>,
    }

    #[derive(Deserialize)]
    struct RuleData {
        word: String,
        #[serde(default)]
        phoneme: Option<String>,
        #[serde(default)]
        aliases: Option<Vec<String>>,
    }

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation/dictionaries/{}/rules",
        input.dictionary_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(ListRulesOutput {
            success: false,
            rules: vec![],
            total_count: 0,
            error: Some(error),
        });
    }

    let result: RulesResponse = response.json().await?;

    let rules: Vec<PronunciationRule> = result
        .rules
        .iter()
        .map(|r| PronunciationRule {
            word: r.word.clone(),
            phoneme: r.phoneme.clone(),
            aliases: r.aliases.clone(),
        })
        .collect();

    let total = rules.len();

    Ok(ListRulesOutput {
        success: true,
        rules,
        total_count: total,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn add_rules_ex(api_key: &str, input: AddRulesInput) -> Result<AddRulesOutput> {
    use reqwest::Client;

    let client = Client::new();

    let rules_content = std::fs::read_to_string(&input.rules_file)
        .map_err(|e| anyhow::anyhow!("Failed to read rules file: {}", e))?;

    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation/dictionaries/{}/rules",
        input.dictionary_id
    );

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .body(rules_content)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(AddRulesOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(AddRulesOutput {
        success: true,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn remove_rules_ex(api_key: &str, input: RemoveRulesInput) -> Result<RemoveRulesOutput> {
    use reqwest::Client;

    let client = Client::new();

    let rules_content = std::fs::read_to_string(&input.rules_file)
        .map_err(|e| anyhow::anyhow!("Failed to read rules file: {}", e))?;

    let url = format!(
        "https://api.elevenlabs.io/v1/pronunciation/dictionaries/{}/rules",
        input.dictionary_id
    );

    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .body(rules_content)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(RemoveRulesOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(RemoveRulesOutput {
        success: true,
        error: None,
    })
}

// ============================================================================
// History Handlers (Additional)
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn get_history_item_ex(
    api_key: &str,
    input: GetHistoryItemInput,
) -> Result<GetHistoryItemOutput> {
    use elevenlabs_rs::{endpoints::admin::history::GetHistoryItem, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);

    let endpoint = GetHistoryItem::new(&input.history_item_id);
    let item = client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(GetHistoryItemOutput {
        success: true,
        history_item: Some(HistoryItemInfo {
            history_item_id: item.history_item_id,
            voice_name: item.voice_name,
            voice_id: item.voice_id,
            model_id: item.model_id,
            text: item.text.chars().take(200).collect(),
            date_unix: item.date_unix as i64,
            character_count: item.character_count_change_from as i32,
        }),
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn delete_history_item_ex(
    api_key: &str,
    input: DeleteHistoryItemInput,
) -> Result<DeleteHistoryItemOutput> {
    use elevenlabs_rs::{endpoints::admin::history::DeleteHistoryItem, ElevenLabsClient};

    let client = ElevenLabsClient::new(api_key);

    let endpoint = DeleteHistoryItem::new(&input.history_item_id);
    client
        .hit(endpoint)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(DeleteHistoryItemOutput {
        success: true,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn submit_feedback(
    api_key: &str,
    input: SubmitFeedbackInput,
) -> Result<SubmitFeedbackOutput> {
    use reqwest::Client;
    use serde_json::json;

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/history/{}/feedback",
        input.history_item_id
    );

    let body = if let Some(feedback_text) = input.feedback {
        json!({
            "thumbs_up": input.thumbs_up,
            "feedback": feedback_text
        })
    } else {
        json!({
            "thumbs_up": input.thumbs_up
        })
    };

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(SubmitFeedbackOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(SubmitFeedbackOutput {
        success: true,
        error: None,
    })
}

// ============================================================================
// RAG Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn rebuild_index_ex(
    api_key: &str,
    input: RebuildIndexInput,
) -> Result<RebuildIndexOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct RebuildResponse {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        status: Option<String>,
    }

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/convai/knowledge-base/{}/rebuild-index",
        input.document_id
    );

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(RebuildIndexOutput {
            success: false,
            index_id: None,
            status: None,
            error: Some(error),
        });
    }

    let result: RebuildResponse = response.json().await?;

    Ok(RebuildIndexOutput {
        success: true,
        index_id: result.id,
        status: result.status,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn get_index_status_ex(
    api_key: &str,
    input: GetIndexStatusInput,
) -> Result<GetIndexStatusOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct IndexStatusResponse {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        active: Option<bool>,
        #[serde(default)]
        document_id: Option<String>,
        #[serde(default)]
        error: Option<String>,
    }

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/convai/knowledge-base/{}/index-status",
        input.document_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(GetIndexStatusOutput {
            success: false,
            status: None,
            error: Some(error),
        });
    }

    let result: IndexStatusResponse = response.json().await?;

    Ok(GetIndexStatusOutput {
        success: true,
        status: Some(IndexStatusInfo {
            id: result.id,
            status: result.status,
            active: result.active,
            document_id: result.document_id,
            error: result.error,
        }),
        error: None,
    })
}

// ============================================================================
// Audio Native Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn list_audio_native_ex(
    api_key: &str,
    input: ListAudioNativeInput,
) -> Result<ListAudioNativeOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct AudioNativeListResponse {
        projects: Vec<AudioNativeProject>,
    }

    #[derive(Deserialize)]
    struct AudioNativeProject {
        project_id: String,
        name: String,
        #[serde(default)]
        author: Option<String>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        voice_id: Option<String>,
        #[serde(default)]
        model_id: Option<String>,
        created_at: String,
    }

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/audio-native?limit={}&page={}",
        input.limit, input.page
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(ListAudioNativeOutput {
            success: false,
            projects: vec![],
            total_count: 0,
            error: Some(error),
        });
    }

    let result: AudioNativeListResponse = response.json().await?;

    let projects: Vec<AudioNativeProjectInfo> = result
        .projects
        .iter()
        .map(|p| AudioNativeProjectInfo {
            project_id: p.project_id.clone(),
            name: p.name.clone(),
            author: p.author.clone(),
            title: p.title.clone(),
            voice_id: p.voice_id.clone(),
            model_id: p.model_id.clone(),
            created_at: p.created_at.clone(),
        })
        .collect();

    let total = projects.len();

    Ok(ListAudioNativeOutput {
        success: true,
        projects,
        total_count: total,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn get_audio_native_ex(
    api_key: &str,
    input: GetAudioNativeInput,
) -> Result<GetAudioNativeOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct AudioNativeProject {
        project_id: String,
        name: String,
        #[serde(default)]
        author: Option<String>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        voice_id: Option<String>,
        #[serde(default)]
        model_id: Option<String>,
        created_at: String,
    }

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/audio-native/{}",
        input.project_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(GetAudioNativeOutput {
            success: false,
            project: None,
            error: Some(error),
        });
    }

    let project: AudioNativeProject = response.json().await?;

    Ok(GetAudioNativeOutput {
        success: true,
        project: Some(AudioNativeProjectInfo {
            project_id: project.project_id,
            name: project.name,
            author: project.author,
            title: project.title,
            voice_id: project.voice_id,
            model_id: project.model_id,
            created_at: project.created_at,
        }),
        error: None,
    })
}

// ============================================================================
// Conversation Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn get_conversation_ex(
    api_key: &str,
    input: GetConversationInput,
) -> Result<GetConversationOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TranscriptData {
        role: String,
        content: String,
        #[serde(default)]
        timestamp: Option<String>,
    }

    #[derive(Deserialize)]
    struct ConversationDetail {
        conversation_id: String,
        #[serde(default)]
        agent_id: Option<String>,
        #[serde(default)]
        version_id: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        created_at: Option<String>,
        #[serde(default)]
        transcript: Option<Vec<TranscriptData>>,
    }

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/convai/conversations/{}",
        input.conversation_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(GetConversationOutput {
            success: false,
            conversation: None,
            error: Some(error),
        });
    }

    let conv: ConversationDetail = response.json().await?;

    let transcript = conv.transcript.map(|msgs| {
        msgs.into_iter()
            .map(|m| TranscriptMessage {
                role: m.role,
                content: m.content,
                timestamp: m.timestamp,
            })
            .collect()
    });

    Ok(GetConversationOutput {
        success: true,
        conversation: Some(ConversationDetailInfo {
            conversation_id: conv.conversation_id,
            agent_id: conv.agent_id,
            version_id: conv.version_id,
            status: conv.status,
            created_at: conv.created_at,
            transcript,
        }),
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn delete_conversation_ex(
    api_key: &str,
    input: DeleteConversationInput,
) -> Result<DeleteConversationOutput> {
    use reqwest::Client;

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/convai/conversations/{}",
        input.conversation_id
    );

    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(DeleteConversationOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(DeleteConversationOutput {
        success: true,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn get_conversation_audio_ex(
    api_key: &str,
    input: GetConversationAudioInput,
) -> Result<GetConversationAudioOutput> {
    use reqwest::Client;

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/convai/conversations/{}/audio",
        input.conversation_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(GetConversationAudioOutput {
            success: false,
            output_file: None,
            audio_base64: None,
            error: Some(error),
        });
    }

    let audio = response
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if let Some(output_file) = input.output {
        std::fs::write(&output_file, &audio)?;
        Ok(GetConversationAudioOutput {
            success: true,
            output_file: Some(output_file),
            audio_base64: None,
            error: None,
        })
    } else {
        let audio_base64 = BASE64.encode(&audio);
        Ok(GetConversationAudioOutput {
            success: true,
            output_file: None,
            audio_base64: Some(audio_base64),
            error: None,
        })
    }
}

// ============================================================================
// Workspace Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn list_members(api_key: &str, _input: ListMembersInput) -> Result<ListMembersOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct MemberData {
        user_id: String,
        email: String,
        role: String,
        #[serde(default)]
        joined_at: Option<String>,
    }

    let client = Client::new();

    let response = client
        .get("https://api.elevenlabs.io/v1/workspace/members")
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(ListMembersOutput {
            success: false,
            members: vec![],
            total_count: 0,
            error: Some(error),
        });
    }

    let members: Vec<MemberData> = response.json().await?;

    let member_infos: Vec<MemberInfo> = members
        .iter()
        .map(|m| MemberInfo {
            user_id: m.user_id.clone(),
            email: m.email.clone(),
            role: m.role.clone(),
            joined_at: m.joined_at.clone(),
        })
        .collect();

    let total = member_infos.len();

    Ok(ListMembersOutput {
        success: true,
        members: member_infos,
        total_count: total,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn invite_member(api_key: &str, input: InviteMemberInput) -> Result<InviteMemberOutput> {
    use reqwest::Client;
    use serde_json::json;

    let client = Client::new();

    let body = json!({
        "email": input.email,
        "role": input.role.to_lowercase()
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/workspace/invites")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(InviteMemberOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(InviteMemberOutput {
        success: true,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn revoke_invite(api_key: &str, input: RevokeInviteInput) -> Result<RevokeInviteOutput> {
    use reqwest::Client;

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/workspace/invites/{}",
        input.email
    );

    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(RevokeInviteOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(RevokeInviteOutput {
        success: true,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn list_secrets(api_key: &str, _input: ListSecretsInput) -> Result<ListSecretsOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct SecretData {
        name: String,
        secret_type: String,
        #[serde(default)]
        created_at: Option<String>,
    }

    let client = Client::new();

    let response = client
        .get("https://api.elevenlabs.io/v1/convai/workspaces/secrets")
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(ListSecretsOutput {
            success: false,
            secrets: vec![],
            total_count: 0,
            error: Some(error),
        });
    }

    let secrets: Vec<SecretData> = response.json().await?;

    let secret_infos: Vec<SecretInfo> = secrets
        .iter()
        .map(|s| SecretInfo {
            name: s.name.clone(),
            secret_type: s.secret_type.clone(),
            created_at: s.created_at.clone(),
        })
        .collect();

    let total = secret_infos.len();

    Ok(ListSecretsOutput {
        success: true,
        secrets: secret_infos,
        total_count: total,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn add_secret(api_key: &str, input: AddSecretInput) -> Result<AddSecretOutput> {
    use reqwest::Client;
    use serde_json::json;

    let client = Client::new();

    let body = json!({
        "name": input.name,
        "value": input.value,
        "secret_type": input.secret_type.to_lowercase()
    });

    let response = client
        .post("https://api.elevenlabs.io/v1/convai/workspaces/secrets")
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(AddSecretOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(AddSecretOutput {
        success: true,
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn delete_secret(api_key: &str, input: DeleteSecretInput) -> Result<DeleteSecretOutput> {
    use reqwest::Client;

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/convai/workspaces/secrets/{}",
        input.name
    );

    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(DeleteSecretOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(DeleteSecretOutput {
        success: true,
        error: None,
    })
}

// ============================================================================
// Phone Handlers
// ============================================================================

#[cfg(feature = "mcp")]
pub async fn get_phone_number_ex(
    api_key: &str,
    input: GetPhoneNumberInput,
) -> Result<GetPhoneNumberOutput> {
    use reqwest::Client;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct PhoneNumberData {
        phone_number_id: String,
        phone_number: String,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        provider: Option<String>,
        #[serde(default)]
        agent_id: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        created_at: Option<String>,
    }

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/convai/phone-numbers/{}",
        input.phone_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(GetPhoneNumberOutput {
            success: false,
            phone_number: None,
            error: Some(error),
        });
    }

    let phone: PhoneNumberData = response.json().await?;

    Ok(GetPhoneNumberOutput {
        success: true,
        phone_number: Some(PhoneNumberInfo {
            phone_number_id: phone.phone_number_id,
            phone_number: phone.phone_number,
            label: phone.label,
            provider: phone.provider,
            agent_id: phone.agent_id,
            status: phone.status,
            created_at: phone.created_at,
        }),
        error: None,
    })
}

#[cfg(feature = "mcp")]
pub async fn delete_phone_number_ex(
    api_key: &str,
    input: DeletePhoneNumberInput,
) -> Result<DeletePhoneNumberOutput> {
    use reqwest::Client;

    let client = Client::new();
    let url = format!(
        "https://api.elevenlabs.io/v1/convai/phone-numbers/{}",
        input.phone_id
    );

    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Ok(DeletePhoneNumberOutput {
            success: false,
            error: Some(error),
        });
    }

    Ok(DeletePhoneNumberOutput {
        success: true,
        error: None,
    })
}
