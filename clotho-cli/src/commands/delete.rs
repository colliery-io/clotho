use chrono::Utc;
use clap::Args;

use clotho_core::domain::types::EntityId;
use clotho_core::graph::GraphStore;
use clotho_store::data::entities::EntityStore;
use clotho_store::data::jsonl::{Event, EventStore, EventType};
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct DeleteArgs {
    /// Entity ID (UUID) to delete.
    pub id: String,
}

pub fn run(args: DeleteArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let store = EntityStore::open(&ws.data_path().join("entities.db"))?;

    // Verify entity exists and get its content path
    let row = store
        .get(&args.id)?
        .ok_or_else(|| format!("Entity not found: {}", args.id))?;

    let title = row.title.clone();

    // Delete content file if it exists
    if let Some(ref path) = row.content_path {
        let p = std::path::Path::new(path);
        if p.exists() {
            std::fs::remove_file(p)?;
        }
    }

    // Delete from entities.db
    store.delete(&args.id)?;

    // Delete from graph
    if let Ok(graph) = GraphStore::open(&ws.graph_path().join("relations.db")) {
        let id: EntityId = uuid::Uuid::parse_str(&args.id)
            .map(EntityId::from)
            .map_err(|e| format!("invalid ID: {}", e))?;
        let _ = graph.remove_node(&id);
    }

    // Delete from search index
    if let Ok(index) = SearchIndex::open(&ws.index_path().join("search.db")) {
        let _ = index.remove_entity(&args.id);
    }

    // Log event
    let events = EventStore::new(&ws.data_path());
    events.log(&Event {
        timestamp: Utc::now(),
        event_type: EventType::Deleted,
        entity_id: args.id.clone(),
        details: None,
    })?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "id": args.id,
            "deleted": true,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Deleted {} ({})", title, args.id);
    }

    Ok(())
}
