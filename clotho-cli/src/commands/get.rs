use clap::Args;

use clotho_store::data::entities::EntityStore;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct GetArgs {
    /// Entity ID (UUID) to read.
    pub id: String,
}

pub fn run(args: GetArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let store = EntityStore::open(&ws.data_path().join("entities.db"))?;

    let row = store
        .get(&args.id)?
        .ok_or_else(|| format!("Entity not found: {}", args.id))?;

    if json {
        let out = serde_json::json!({
            "id": row.id,
            "entity_type": row.entity_type,
            "title": row.title,
            "created_at": row.created_at,
            "updated_at": row.updated_at,
            "status": row.status,
            "task_state": row.task_state,
            "extraction_status": row.extraction_status,
            "confidence": row.confidence,
            "content_path": row.content_path,
            "metadata": row.metadata,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("{} ({})", row.title, row.entity_type);
        println!("{}", "-".repeat(60));
        println!("  ID:       {}", row.id);
        println!("  Type:     {}", row.entity_type);
        println!("  Created:  {}", row.created_at);
        println!("  Updated:  {}", row.updated_at);
        if let Some(ref s) = row.status {
            println!("  Status:   {}", s);
        }
        if let Some(ref s) = row.task_state {
            println!("  State:    {}", s);
        }
        if let Some(ref s) = row.extraction_status {
            println!("  Extract:  {}", s);
        }
        if let Some(c) = row.confidence {
            println!("  Confidence: {:.2}", c);
        }

        // Try to read content
        if let Some(ref path) = row.content_path {
            let p = std::path::Path::new(path);
            if p.exists() {
                let content = std::fs::read_to_string(p)?;
                println!("\n{}", content);
            }
        }
    }

    Ok(())
}
