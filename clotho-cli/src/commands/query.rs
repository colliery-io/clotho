use clap::Args;

use clotho_core::graph::GraphStore;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct QueryArgs {
    /// Cypher query to execute against the relation graph.
    pub cypher: String,
}

pub fn run(args: QueryArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
        .map_err(|e| format!("graph error: {}", e))?;

    let result = graph
        .raw_cypher(&args.cypher)
        .map_err(|e| format!("query error: {}", e))?;

    if json {
        let mut rows = Vec::new();
        let columns = result.columns().to_vec();
        for row in result.iter() {
            let mut map = serde_json::Map::new();
            for col in &columns {
                let val: String = row.get(col).unwrap_or_default();
                map.insert(col.clone(), serde_json::Value::String(val));
            }
            rows.push(serde_json::Value::Object(map));
        }
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        if result.is_empty() {
            println!("No results.");
            return Ok(());
        }

        let columns = result.columns().to_vec();

        // Print header
        let header: Vec<String> = columns.iter().map(|c| format!("{:<20}", c)).collect();
        println!("{}", header.join(" "));
        println!("{}", "-".repeat(columns.len() * 21));

        // Print rows
        for row in result.iter() {
            let vals: Vec<String> = columns
                .iter()
                .map(|col| {
                    let val: String = row.get(col).unwrap_or_default();
                    format!(
                        "{:<20}",
                        if val.len() > 18 {
                            format!("{}...", &val[..15])
                        } else {
                            val
                        }
                    )
                })
                .collect();
            println!("{}", vals.join(" "));
        }

        println!("\n{} rows", result.len());
    }

    Ok(())
}
