use crate::cli::{VoiceLibraryArgs, VoiceLibraryCommands};
use crate::client::create_http_client;
use crate::output::{print_info, print_success};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use elevenlabs_rs::{
    endpoints::admin::voice_library::{
        AddSharedVoice, Age, Gender, GetSharedVoices, Language, SharedVoiceCategory,
        SharedVoicesQuery,
    },
    ElevenLabsClient,
};
use serde::Deserialize;

pub async fn execute(args: VoiceLibraryArgs, api_key: &str) -> Result<()> {
    match args.command {
        VoiceLibraryCommands::List {
            page_size,
            category,
            gender,
            age,
            language,
            accent,
            use_cases,
            descriptives,
            search,
            featured,
        } => {
            let client = ElevenLabsClient::new(api_key);
            list_shared_voices(
                &client,
                page_size,
                category,
                gender,
                age,
                language,
                accent,
                use_cases,
                descriptives,
                search,
                featured,
            )
            .await?
        }
        VoiceLibraryCommands::Saved { page_size } => list_saved_voices(api_key, page_size).await?,
        VoiceLibraryCommands::Add {
            public_user_id,
            voice_id,
            name,
        } => {
            let client = ElevenLabsClient::new(api_key);
            add_shared_voice(&client, &public_user_id, &voice_id, &name).await?
        }
        VoiceLibraryCommands::Collections { page_size } => {
            list_collections(api_key, page_size).await?
        }
        VoiceLibraryCommands::CollectionVoices { collection_id } => {
            list_collection_voices(api_key, &collection_id).await?
        }
    }

    Ok(())
}

async fn list_saved_voices(api_key: &str, page_size: Option<u32>) -> Result<()> {
    let client = create_http_client();
    print_info("Fetching saved voices...");

    let mut url = "https://api.elevenlabs.io/v2/voices?voice_type=saved".to_string();
    if let Some(ps) = page_size {
        url.push_str(&format!("&page_size={}", ps));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch saved voices")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct SavedVoicesResponse {
        voices: Vec<SavedVoice>,
    }

    #[derive(Deserialize)]
    struct SavedVoice {
        voice_id: String,
        name: String,
        #[serde(default)]
        category: Option<String>,
        #[serde(default)]
        collection_ids: Option<Vec<String>>,
    }

    let result: SavedVoicesResponse = response.json().await.context("Failed to parse response")?;

    if result.voices.is_empty() {
        print_info("No saved voices found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Name", "Voice ID", "Category", "Collections"]);

    for voice in &result.voices {
        let collections = voice
            .collection_ids
            .as_ref()
            .map(|ids| ids.len().to_string())
            .unwrap_or_else(|| "0".to_string());

        table.add_row(vec![
            voice.name.cyan(),
            voice.voice_id.yellow(),
            voice.category.as_deref().unwrap_or("-").into(),
            collections.into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} saved voice(s)", result.voices.len()));
    Ok(())
}

async fn list_collections(api_key: &str, page_size: Option<u32>) -> Result<()> {
    let client = create_http_client();
    print_info("Fetching voice collections...");

    let mut url = "https://api.elevenlabs.io/v1/voices/collections".to_string();
    if let Some(ps) = page_size {
        url.push_str(&format!("?page_size={}", ps));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch collections")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct CollectionsResponse {
        collections: Vec<CollectionInfo>,
    }

    #[derive(Deserialize)]
    struct CollectionInfo {
        collection_id: String,
        name: String,
        #[serde(default)]
        voice_count: Option<u32>,
        #[serde(default)]
        created_at: Option<String>,
    }

    let result: CollectionsResponse = response.json().await.context("Failed to parse response")?;

    if result.collections.is_empty() {
        print_info("No collections found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Collection ID", "Name", "Voices", "Created"]);

    for collection in &result.collections {
        table.add_row(vec![
            collection.collection_id.yellow(),
            collection.name.cyan(),
            collection.voice_count.unwrap_or(0).to_string().into(),
            collection.created_at.as_deref().unwrap_or("-").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} collection(s)", result.collections.len()));
    Ok(())
}

async fn list_collection_voices(api_key: &str, collection_id: &str) -> Result<()> {
    let client = create_http_client();
    print_info(&format!(
        "Fetching voices in collection '{}'...",
        collection_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/voices/collections/{}/voices",
        collection_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch collection voices")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct CollectionVoicesResponse {
        voices: Vec<CollectionVoice>,
    }

    #[derive(Deserialize)]
    struct CollectionVoice {
        voice_id: String,
        name: String,
        #[serde(default)]
        category: Option<String>,
    }

    let result: CollectionVoicesResponse =
        response.json().await.context("Failed to parse response")?;

    if result.voices.is_empty() {
        print_info("No voices in this collection");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec!["Name", "Voice ID", "Category"]);

    for voice in &result.voices {
        table.add_row(vec![
            voice.name.cyan(),
            voice.voice_id.yellow(),
            voice.category.as_deref().unwrap_or("-").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!(
        "Found {} voice(s) in collection",
        result.voices.len()
    ));
    Ok(())
}

async fn list_shared_voices(
    client: &ElevenLabsClient,
    page_size: Option<u32>,
    category: Option<String>,
    gender: Option<String>,
    age: Option<String>,
    language: Option<String>,
    accent: Option<String>,
    use_cases: Option<String>,
    descriptives: Option<String>,
    search: Option<String>,
    featured: bool,
) -> Result<()> {
    print_info("Fetching shared voices from the library...");

    let mut query = SharedVoicesQuery::default();

    if let Some(ps) = page_size {
        query = query.with_page_size(ps as u16);
    }

    if let Some(cat) = category {
        let cat_enum = match cat.to_lowercase().as_str() {
            "professional" => SharedVoiceCategory::Professional,
            "high_quality" | "highquality" => SharedVoiceCategory::HighQuality,
            "generated" => SharedVoiceCategory::Generated,
            "famous" => SharedVoiceCategory::Famous,
            _ => SharedVoiceCategory::HighQuality,
        };
        query = query.with_category(cat_enum);
    }

    if let Some(g) = gender {
        let gender_enum = match g.to_lowercase().as_str() {
            "male" => Gender::Male,
            "female" => Gender::Female,
            _ => Gender::Female,
        };
        query = query.with_gender(gender_enum);
    }

    if let Some(a) = age {
        let age_enum = match a.to_lowercase().as_str() {
            "young" => Age::Young,
            "middle_aged" | "middleaged" | "middle-aged" => Age::MiddleAged,
            "old" => Age::Old,
            _ => Age::Young,
        };
        query = query.with_age(age_enum);
    }

    if let Some(_lang) = language {
        // Language conversion from string not directly supported
        query = query.with_language(Language::English);
    }

    if let Some(acc) = accent {
        query = query.with_accent(&acc);
    }

    if let Some(uc) = use_cases {
        query = query.with_use_cases(&uc);
    }

    if let Some(desc) = descriptives {
        query = query.with_descriptives(&desc);
    }

    if let Some(s) = search {
        query = query.with_search(&s);
    }

    if featured {
        query = query.with_featured(true);
    }

    let endpoint = GetSharedVoices::with_query(query);
    let response = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    if response.voices.is_empty() {
        print_info("No shared voices found with the given criteria");
        return Ok(());
    }

    println!("\n{}", "Shared Voices:".bold().underline());

    let mut table = Table::new();
    table.set_header(vec![
        "Name", "Voice ID", "Owner", "Category", "Gender", "Age",
    ]);

    for voice in &response.voices {
        table.add_row(vec![
            voice.name.clone(),
            voice.voice_id.clone(),
            voice.public_owner_id.chars().take(12).collect::<String>() + "...",
            format!("{:?}", voice.category),
            format!("{:?}", voice.gender),
            format!("{:?}", voice.age),
        ]);
    }

    println!("{}", table);
    print_success(&format!("Found {} shared voices", response.voices.len()));

    Ok(())
}

async fn add_shared_voice(
    client: &ElevenLabsClient,
    public_user_id: &str,
    voice_id: &str,
    name: &str,
) -> Result<()> {
    print_info(&format!("Adding shared voice '{}'...", name.cyan()));

    let endpoint = AddSharedVoice::new(public_user_id, voice_id, name);
    let response = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;

    print_success(&format!("Added voice '{}' to your library", name));
    println!("  Voice ID: {}", response.voice_id.cyan());

    Ok(())
}
