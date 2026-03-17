use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use clap::Args;

use clotho_core::domain::types::{EntityId, EntityType};
use clotho_core::graph::GraphStore;
use clotho_store::content::ContentStore;
use clotho_store::data::entities::{EntityRow, EntityStore};
use clotho_store::data::jsonl::EventStore;
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct IngestArgs {
    /// Path to the file to ingest.
    pub file: PathBuf,

    /// Entity type for the ingested content.
    #[arg(long, default_value = "note")]
    pub r#type: String,

    /// Title for the entity (defaults to filename).
    #[arg(long)]
    pub title: Option<String>,
}

pub fn run(args: IngestArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Validate file exists
    if !args.file.exists() {
        return Err(format!("File not found: {}", args.file.display()).into());
    }

    // Open workspace from current directory
    let ws = Workspace::open(&std::env::current_dir()?)?;

    // Parse entity type
    let entity_type = parse_entity_type(&args.r#type)?;

    // Read file content
    let content = fs::read_to_string(&args.file)?;

    // Determine title
    let title = args.title.unwrap_or_else(|| {
        args.file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string()
    });

    let id = EntityId::new();
    let now = Utc::now();

    // Store content file
    let content_store = ContentStore::new(&ws.project_root());
    let content_path = content_store.write_content(entity_type, &id, &content)?;

    // Insert entity row
    let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))?;
    let row = EntityRow {
        id: id.to_string(),
        entity_type: format!("{}", entity_type),
        title: title.clone(),
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        status: Some("active".to_string()),
        task_state: None,
        extraction_status: None,
        source_transcript_id: None,
        source_span_start: None,
        source_span_end: None,
        confidence: None,
        content_path: Some(content_path.display().to_string()),
        metadata: None,
    };
    entity_store.insert(&row)?;

    // Register graph node
    let graph_store = GraphStore::open(&ws.graph_path().join("relations.db"))?;
    graph_store
        .register_node(&id, entity_type, &title)
        .map_err(|e| format!("graph error: {}", e))?;

    // Index in FTS5
    let search_index = SearchIndex::open(&ws.index_path().join("search.db"))?;
    search_index.index_entity(&id.to_string(), &format!("{}", entity_type), &title, &content)?;

    // Log event
    let event_store = EventStore::new(&ws.data_path());
    event_store.log(&clotho_store::data::jsonl::Event {
        timestamp: now,
        event_type: clotho_store::data::jsonl::EventType::Created,
        entity_id: id.to_string(),
        details: Some(serde_json::json!({"source_file": args.file.display().to_string()})),
    })?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "id": id.to_string(),
            "entity_type": format!("{}", entity_type),
            "title": title,
            "content_path": content_path.display().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Ingested {} as {} ({})", args.file.display(), title, entity_type);
        println!("  ID: {}", id);
        println!("  Content: {}", content_path.display());
    }

    Ok(())
}

fn parse_entity_type(s: &str) -> Result<EntityType, Box<dyn std::error::Error>> {
    match s.to_lowercase().as_str() {
        "note" => Ok(EntityType::Note),
        "meeting" => Ok(EntityType::Meeting),
        "transcript" => Ok(EntityType::Transcript),
        "artifact" => Ok(EntityType::Artifact),
        "reflection" => Ok(EntityType::Reflection),
        _ => Err(format!(
            "Unknown entity type '{}'. Valid types: note, meeting, transcript, artifact, reflection",
            s
        )
        .into()),
    }
}
