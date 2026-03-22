use chrono::Utc;
use clap::Args;

use clotho_store::data::entities::EntityStore;
use clotho_store::data::jsonl::{Event, EventStore, EventType};
use clotho_store::workspace::Workspace;

use crate::resolve;

#[derive(Args)]
pub struct UpdateArgs {
    /// Entity ID (full UUID or prefix).
    pub id: String,

    /// New title.
    #[arg(long)]
    pub title: Option<String>,

    /// New status (active, inactive).
    #[arg(long)]
    pub status: Option<String>,

    /// New task state (todo, doing, blocked, done).
    #[arg(long)]
    pub state: Option<String>,
}

pub fn run(args: UpdateArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let store = EntityStore::open(&ws.data_path().join("entities.db"))?;

    let mut row = resolve::resolve_for_write(&store, &args.id)?;

    // Apply updates
    if let Some(ref title) = args.title {
        row.title = title.clone();
    }
    if let Some(ref status) = args.status {
        row.status = Some(status.clone());
    }
    if let Some(ref state) = args.state {
        row.task_state = Some(state.clone());
    }
    row.updated_at = Utc::now().to_rfc3339();

    store.update(&row)?;

    // Update graph node title if changed
    if args.title.is_some() {
        if let Ok(graph) =
            clotho_core::graph::GraphStore::open(&ws.graph_path().join("relations.db"))
        {
            if let Ok(et) = parse_entity_type_str(&row.entity_type) {
                let id = uuid::Uuid::parse_str(&row.id)
                    .map(clotho_core::domain::types::EntityId::from)?;
                let _ = graph.register_node(&id, et, &row.title);
            }
        }
    }

    // Update FTS5 index
    if let Ok(index) = clotho_store::index::SearchIndex::open(&ws.index_path().join("search.db")) {
        let content = row
            .content_path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default();
        let _ = index.index_entity(&row.id, &row.entity_type, &row.title, &content);
    }

    // Log event
    let events = EventStore::new(&ws.data_path());
    events.log(&Event {
        timestamp: Utc::now(),
        event_type: EventType::Updated,
        entity_id: row.id.clone(),
        details: None,
    })?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "id": row.id,
            "title": row.title,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Updated entity {}", row.id);
    }

    Ok(())
}

fn parse_entity_type_str(s: &str) -> Result<clotho_core::domain::types::EntityType, String> {
    use clotho_core::domain::types::EntityType;
    match s {
        "Program" => Ok(EntityType::Program),
        "Responsibility" => Ok(EntityType::Responsibility),
        "Objective" => Ok(EntityType::Objective),
        "Workstream" => Ok(EntityType::Workstream),
        "Task" => Ok(EntityType::Task),
        "Meeting" => Ok(EntityType::Meeting),
        "Transcript" => Ok(EntityType::Transcript),
        "Note" => Ok(EntityType::Note),
        "Reflection" => Ok(EntityType::Reflection),
        "Artifact" => Ok(EntityType::Artifact),
        "Reference" => Ok(EntityType::Reference),
        "Decision" => Ok(EntityType::Decision),
        "Risk" => Ok(EntityType::Risk),
        "Blocker" => Ok(EntityType::Blocker),
        "Question" => Ok(EntityType::Question),
        "Insight" => Ok(EntityType::Insight),
        "Person" => Ok(EntityType::Person),
        _ => Err(format!("unknown type: {}", s)),
    }
}
