use chrono::Utc;
use clap::Args;

use clotho_core::domain::types::{EntityId, EntityType, PeriodType};
use clotho_core::graph::GraphStore;
use clotho_store::content::ContentStore;
use clotho_store::data::entities::{EntityRow, EntityStore};
use clotho_store::data::jsonl::{Event, EventStore, EventType};
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct ReflectArgs {
    /// Period type for the reflection.
    #[arg(long)]
    pub period: String,

    /// Optional title (defaults to period-based name).
    #[arg(long)]
    pub title: Option<String>,

    /// Optional program ID to scope the reflection to.
    #[arg(long)]
    pub program: Option<String>,
}

pub fn run(args: ReflectArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let period_type = parse_period_type(&args.period)?;
    let now = Utc::now();
    let id = EntityId::new();

    // Generate title
    let title = args.title.unwrap_or_else(|| {
        let period_str = match period_type {
            PeriodType::Daily => now.format("%Y-%m-%d daily").to_string(),
            PeriodType::Weekly => now.format("%Y-W%V weekly").to_string(),
            PeriodType::Monthly => now.format("%Y-%m monthly").to_string(),
            PeriodType::Quarterly => {
                let q = (now.format("%m").to_string().parse::<u32>().unwrap_or(1) - 1) / 3 + 1;
                format!("{}-Q{} quarterly", now.format("%Y"), q)
            }
            PeriodType::Adhoc => format!("{} reflection", now.format("%Y-%m-%d")),
        };
        format!("{} reflection", period_str)
    });

    // Generate content template
    let template = format!(
        "# {}\n\n## Period\n\nType: {}\nDate: {}\n\n## Reflections\n\n\n\n## Key Takeaways\n\n\n\n## Action Items\n\n\n",
        title,
        args.period,
        now.format("%Y-%m-%d"),
    );

    // Write content
    let content_store = ContentStore::new(&ws.path);
    let content_path = content_store.write_content(EntityType::Reflection, &id, &template)?;

    // Insert entity row
    let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))?;
    let mut metadata = serde_json::Map::new();
    metadata.insert("period_type".to_string(), serde_json::Value::String(args.period.clone()));
    if let Some(ref prog) = args.program {
        metadata.insert("program_id".to_string(), serde_json::Value::String(prog.clone()));
    }

    let row = EntityRow {
        id: id.to_string(),
        entity_type: "Reflection".to_string(),
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
        metadata: Some(serde_json::to_string(&metadata)?),
    };
    entity_store.insert(&row)?;

    // Register graph node
    let graph_store = GraphStore::open(&ws.graph_path().join("relations.db"))
        .map_err(|e| format!("graph error: {}", e))?;
    graph_store
        .register_node(&id, EntityType::Reflection, &title)
        .map_err(|e| format!("graph error: {}", e))?;

    // Index in FTS5
    let search_index = SearchIndex::open(&ws.index_path().join("search.db"))?;
    search_index.index_entity(&id.to_string(), "Reflection", &title, &template)?;

    // Log event
    let event_store = EventStore::new(&ws.data_path());
    event_store.log(&Event {
        timestamp: now,
        event_type: EventType::Created,
        entity_id: id.to_string(),
        details: Some(serde_json::json!({"period": args.period})),
    })?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "id": id.to_string(),
            "title": title,
            "period": args.period,
            "content_path": content_path.display().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Created {} reflection: {}", args.period, title);
        println!("  ID: {}", id);
        println!("  File: {}", content_path.display());
    }

    Ok(())
}

fn parse_period_type(s: &str) -> Result<PeriodType, Box<dyn std::error::Error>> {
    match s.to_lowercase().as_str() {
        "daily" => Ok(PeriodType::Daily),
        "weekly" => Ok(PeriodType::Weekly),
        "monthly" => Ok(PeriodType::Monthly),
        "quarterly" => Ok(PeriodType::Quarterly),
        "adhoc" => Ok(PeriodType::Adhoc),
        _ => Err(format!(
            "Unknown period '{}'. Valid: daily, weekly, monthly, quarterly, adhoc",
            s
        )
        .into()),
    }
}
