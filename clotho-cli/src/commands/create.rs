use chrono::Utc;
use clap::Args;

use clotho_core::domain::types::{EntityId, EntityType};
use clotho_core::graph::GraphStore;
use clotho_store::content::ContentStore;
use clotho_store::data::entities::{EntityRow, EntityStore};
use clotho_store::data::jsonl::{Event, EventStore, EventType};
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct CreateArgs {
    /// Entity type to create (program, responsibility, objective, workstream, task, meeting,
    /// transcript, note, reflection, artifact, decision, risk, blocker, question, insight, person).
    pub entity_type: String,

    /// Title of the entity.
    #[arg(long)]
    pub title: String,

    /// Status (active, inactive). Defaults based on entity type.
    #[arg(long)]
    pub status: Option<String>,

    /// Task state (todo, doing, blocked, done). Only for Task entities.
    #[arg(long)]
    pub state: Option<String>,

    /// Email address. Only for Person entities.
    #[arg(long)]
    pub email: Option<String>,

    /// Parent entity ID. Creates a BELONGS_TO relation.
    #[arg(long)]
    pub parent: Option<String>,

    /// Inline content (markdown).
    #[arg(long)]
    pub content: Option<String>,
}

pub fn run(args: CreateArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let entity_type = parse_entity_type(&args.entity_type)?;
    let id = EntityId::new();
    let now = Utc::now();

    // Determine defaults based on entity type
    let (default_status, default_state, default_extraction) = defaults_for_type(entity_type);
    // Build metadata before moving fields
    let metadata = build_metadata(entity_type, &args);

    let status = args.status.or(default_status);
    let task_state = args.state.or(default_state);
    let extraction_status = default_extraction;

    // Write content if provided or if entity is ContentBearing
    let content_store = ContentStore::new(&ws.project_root());
    let content_text = args.content.unwrap_or_default();
    let content_path = if !content_text.is_empty() || is_content_bearing(entity_type) {
        let text = if content_text.is_empty() {
            format!("# {}\n", args.title)
        } else {
            content_text.clone()
        };
        Some(content_store.write_content(entity_type, &id, &text)?)
    } else {
        None
    };

    // Insert entity row
    let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))?;
    let row = EntityRow {
        id: id.to_string(),
        entity_type: format!("{}", entity_type),
        title: args.title.clone(),
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
        status,
        task_state,
        extraction_status,
        source_transcript_id: None,
        source_span_start: None,
        source_span_end: None,
        confidence: None,
        content_path: content_path.as_ref().map(|p| p.display().to_string()),
        metadata,
    };
    entity_store.insert(&row)?;

    // Register graph node
    let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
        .map_err(|e| format!("graph error: {}", e))?;
    graph
        .register_node(&id, entity_type, &args.title)
        .map_err(|e| format!("graph error: {}", e))?;

    // Create BELONGS_TO edge if --parent provided
    if let Some(ref parent_id_str) = args.parent {
        let parent_id: EntityId = uuid::Uuid::parse_str(parent_id_str)
            .map(EntityId::from)
            .map_err(|e| format!("invalid parent ID: {}", e))?;
        graph
            .add_edge(&id, &parent_id, clotho_core::domain::traits::RelationType::BelongsTo)
            .map_err(|e| format!("graph error: {}", e))?;
    }

    // Index in FTS5
    let index = SearchIndex::open(&ws.index_path().join("search.db"))?;
    index.index_entity(
        &id.to_string(),
        &format!("{}", entity_type),
        &args.title,
        &content_text,
    )?;

    // Log event
    let events = EventStore::new(&ws.data_path());
    events.log(&Event {
        timestamp: now,
        event_type: EventType::Created,
        entity_id: id.to_string(),
        details: Some(serde_json::json!({"entity_type": format!("{}", entity_type)})),
    })?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "id": id.to_string(),
            "entity_type": format!("{}", entity_type),
            "title": args.title,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Created {} '{}' ({})", entity_type, args.title, id);
        if let Some(ref parent) = args.parent {
            println!("  Parent: {} (BELONGS_TO)", parent);
        }
    }

    Ok(())
}

fn parse_entity_type(s: &str) -> Result<EntityType, Box<dyn std::error::Error>> {
    match s.to_lowercase().as_str() {
        "program" => Ok(EntityType::Program),
        "responsibility" => Ok(EntityType::Responsibility),
        "objective" => Ok(EntityType::Objective),
        "workstream" => Ok(EntityType::Workstream),
        "task" => Ok(EntityType::Task),
        "meeting" => Ok(EntityType::Meeting),
        "transcript" => Ok(EntityType::Transcript),
        "note" => Ok(EntityType::Note),
        "reflection" => Ok(EntityType::Reflection),
        "artifact" => Ok(EntityType::Artifact),
        "decision" => Ok(EntityType::Decision),
        "risk" => Ok(EntityType::Risk),
        "blocker" => Ok(EntityType::Blocker),
        "question" => Ok(EntityType::Question),
        "insight" => Ok(EntityType::Insight),
        "person" => Ok(EntityType::Person),
        _ => Err(format!(
            "Unknown entity type '{}'. Valid: program, responsibility, objective, workstream, task, meeting, transcript, note, reflection, artifact, decision, risk, blocker, question, insight, person",
            s
        ).into()),
    }
}

/// Returns (default_status, default_task_state, default_extraction_status) for a type.
fn defaults_for_type(et: EntityType) -> (Option<String>, Option<String>, Option<String>) {
    match et {
        // Structural + Workstream: active by default
        EntityType::Program
        | EntityType::Responsibility
        | EntityType::Objective
        | EntityType::Workstream => (Some("active".to_string()), None, None),
        // Task: todo by default
        EntityType::Task => (None, Some("todo".to_string()), None),
        // Derived: draft by default
        EntityType::Decision
        | EntityType::Risk
        | EntityType::Blocker
        | EntityType::Question
        | EntityType::Insight => (None, None, Some("draft".to_string())),
        // Capture + Person: no default status
        _ => (None, None, None),
    }
}

fn is_content_bearing(et: EntityType) -> bool {
    !matches!(
        et,
        EntityType::Decision
            | EntityType::Risk
            | EntityType::Blocker
            | EntityType::Question
            | EntityType::Insight
    )
}

fn build_metadata(et: EntityType, args: &CreateArgs) -> Option<String> {
    let mut meta = serde_json::Map::new();

    if et == EntityType::Person {
        if let Some(ref email) = args.email {
            meta.insert("email".to_string(), serde_json::Value::String(email.clone()));
        }
    }

    if let Some(ref parent) = args.parent {
        meta.insert("parent_id".to_string(), serde_json::Value::String(parent.clone()));
    }

    if meta.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&meta).unwrap_or_default())
    }
}
