use chrono::Utc;
use clap::Args;

use clotho_core::domain::types::EntityId;
use clotho_core::graph::GraphStore;
use clotho_store::data::entities::EntityStore;
use clotho_store::data::jsonl::{Event, EventStore, EventType};
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;

use crate::resolve;

#[derive(Args)]
pub struct DeleteArgs {
    /// Entity ID (full UUID or prefix).
    pub id: String,
}

pub fn run(args: DeleteArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let store = EntityStore::open(&ws.data_path().join("entities.db"))?;

    // Resolve and verify entity exists
    let row = resolve::resolve_for_write(&store, &args.id)?;

    let resolved_id = row.id.clone();
    let title = row.title.clone();

    // Delete content file if it exists
    if let Some(ref path) = row.content_path {
        let p = std::path::Path::new(path);
        if p.exists() {
            std::fs::remove_file(p)?;
        }
    }

    // Delete from entities.db
    store.delete(&resolved_id)?;

    // Delete from graph
    if let Ok(graph) = GraphStore::open(&ws.graph_path().join("relations.db")) {
        let id: EntityId = uuid::Uuid::parse_str(&resolved_id)
            .map(EntityId::from)
            .map_err(|e| format!("invalid ID: {}", e))?;
        let _ = graph.remove_node(&id);
    }

    // Delete from search index
    if let Ok(index) = SearchIndex::open(&ws.index_path().join("search.db")) {
        let _ = index.remove_entity(&resolved_id);
    }

    // Log event
    let events = EventStore::new(&ws.data_path());
    events.log(&Event {
        timestamp: Utc::now(),
        event_type: EventType::Deleted,
        entity_id: resolved_id.clone(),
        details: None,
    })?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "id": resolved_id,
            "deleted": true,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Deleted {} ({})", title, resolved_id);
    }

    Ok(())
}
