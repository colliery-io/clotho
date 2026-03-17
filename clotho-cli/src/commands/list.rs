use clap::Args;

use clotho_store::data::entities::EntityStore;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct ListArgs {
    /// Filter by entity type (e.g., Task, Program, Note).
    #[arg(long)]
    pub r#type: Option<String>,

    /// Filter by status (active, inactive).
    #[arg(long)]
    pub status: Option<String>,

    /// Filter by task state (todo, doing, blocked, done).
    #[arg(long)]
    pub state: Option<String>,
}

pub fn run(args: ListArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let store = EntityStore::open(&ws.data_path().join("entities.db"))?;

    let rows = if let Some(ref t) = args.r#type {
        store.list_by_type(t)?
    } else if let Some(ref s) = args.status {
        store.list_by_status(s)?
    } else if let Some(ref s) = args.state {
        store.list_by_state(s)?
    } else {
        store.list_all()?
    };

    if json {
        let out: Vec<serde_json::Value> = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "entity_type": r.entity_type,
                    "title": r.title,
                    "status": r.status,
                    "task_state": r.task_state,
                    "created_at": r.created_at,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        if rows.is_empty() {
            println!("No entities found.");
            return Ok(());
        }

        // Print table header
        println!(
            "{:<12} {:<16} {:<40} {:<10}",
            "ID", "TYPE", "TITLE", "STATUS"
        );
        println!("{}", "-".repeat(80));

        for row in &rows {
            let id_short = if row.id.len() > 10 {
                &row.id[..10]
            } else {
                &row.id
            };
            let title_display = if row.title.len() > 38 {
                format!("{}...", &row.title[..35])
            } else {
                row.title.clone()
            };
            let status = row
                .task_state
                .as_deref()
                .or(row.status.as_deref())
                .unwrap_or("-");

            println!(
                "{:<12} {:<16} {:<40} {:<10}",
                format!("{}...", id_short),
                row.entity_type,
                title_display,
                status,
            );
        }

        println!("\n{} entities", rows.len());
    }

    Ok(())
}
