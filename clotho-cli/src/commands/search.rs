use clap::Args;

use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct SearchArgs {
    /// Search query (FTS5 keywords).
    pub query: String,

    /// Maximum number of results.
    #[arg(long, default_value = "10")]
    pub limit: usize,
}

pub fn run(args: SearchArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let index = SearchIndex::open(&ws.index_path().join("search.db"))?;

    let mut results = index.search(&args.query)?;
    results.truncate(args.limit);

    if json {
        let out: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "entity_id": r.entity_id,
                    "entity_type": r.entity_type,
                    "title": r.title,
                    "snippet": r.snippet,
                    "rank": r.rank,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        if results.is_empty() {
            println!("No results found for '{}'.", args.query);
            return Ok(());
        }

        for (i, r) in results.iter().enumerate() {
            println!("{}. [{}] {}", i + 1, r.entity_type, r.title);
            println!("   {}", r.snippet);
            println!("   ID: {}", r.entity_id);
            println!();
        }

        println!("{} results", results.len());
    }

    Ok(())
}
