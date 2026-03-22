use clap::Args;

use clotho_store::data::entities::EntityStore;
use clotho_store::data::ontology::{
    OntologyStore, CATEGORY_KEYWORD, CATEGORY_PERSON, CATEGORY_SIGNAL_SOCIAL,
    CATEGORY_SIGNAL_TECHNICAL,
};
use clotho_store::workspace::Workspace;

use crate::resolve;

#[derive(Args)]
pub struct OntologyGetArgs {
    /// Entity ID (full UUID or prefix).
    pub id: String,
}

#[derive(Args)]
pub struct OntologySetArgs {
    /// Entity ID (full UUID or prefix).
    pub id: String,

    /// Add keywords (comma-separated).
    #[arg(long)]
    pub add_keywords: Option<String>,

    /// Remove keywords (comma-separated).
    #[arg(long)]
    pub remove_keywords: Option<String>,

    /// Add technical signal types (comma-separated).
    #[arg(long)]
    pub add_technical: Option<String>,

    /// Add social signal types (comma-separated).
    #[arg(long)]
    pub add_social: Option<String>,

    /// Add involved people (comma-separated).
    #[arg(long)]
    pub add_people: Option<String>,

    /// Remove people (comma-separated).
    #[arg(long)]
    pub remove_people: Option<String>,
}

#[derive(Args)]
pub struct OntologySearchArgs {
    /// Search term to find across all ontologies.
    pub query: String,
}

pub fn run_get(args: OntologyGetArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))?;
    let ontology_store = OntologyStore::open(&ws.data_path().join("entities.db"))?;

    let row = resolve::resolve_for_read(&entity_store, &args.id)?;

    let ontology = ontology_store.get(&row.id)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&ontology)?);
    } else {
        println!("{} — {} ({})", row.title, row.entity_type, &row.id[..8]);
        println!("{}", "-".repeat(60));

        println!("\nKeywords:");
        if ontology.keywords.is_empty() {
            println!("  (none)");
        }
        for k in &ontology.keywords {
            println!("  - {}", k);
        }

        println!("\nSignal types (technical):");
        if ontology.signal_technical.is_empty() {
            println!("  (none)");
        }
        for t in &ontology.signal_technical {
            println!("  - {}", t);
        }

        println!("\nSignal types (social):");
        if ontology.signal_social.is_empty() {
            println!("  (none)");
        }
        for s in &ontology.signal_social {
            println!("  - {}", s);
        }

        println!("\nInvolved people:");
        if ontology.people.is_empty() {
            println!("  (none)");
        }
        for p in &ontology.people {
            println!("  - {}", p);
        }
    }

    Ok(())
}

pub fn run_set(args: OntologySetArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))?;
    let ontology_store = OntologyStore::open(&ws.data_path().join("entities.db"))?;

    let row = resolve::resolve_for_write(&entity_store, &args.id)?;
    let resolved_id = &row.id;

    // Additions
    if let Some(ref kw) = args.add_keywords {
        let vals: Vec<&str> = kw.split(',').collect();
        ontology_store.add(resolved_id, CATEGORY_KEYWORD, &vals, Some("user"))?;
    }
    if let Some(ref ts) = args.add_technical {
        let vals: Vec<&str> = ts.split(',').collect();
        ontology_store.add(resolved_id, CATEGORY_SIGNAL_TECHNICAL, &vals, Some("user"))?;
    }
    if let Some(ref ss) = args.add_social {
        let vals: Vec<&str> = ss.split(',').collect();
        ontology_store.add(resolved_id, CATEGORY_SIGNAL_SOCIAL, &vals, Some("user"))?;
    }
    if let Some(ref pp) = args.add_people {
        let vals: Vec<&str> = pp.split(',').collect();
        ontology_store.add(resolved_id, CATEGORY_PERSON, &vals, Some("user"))?;
    }

    // Removals
    if let Some(ref kw) = args.remove_keywords {
        let vals: Vec<&str> = kw.split(',').collect();
        ontology_store.remove(resolved_id, CATEGORY_KEYWORD, &vals)?;
    }
    if let Some(ref pp) = args.remove_people {
        let vals: Vec<&str> = pp.split(',').collect();
        ontology_store.remove(resolved_id, CATEGORY_PERSON, &vals)?;
    }

    if json {
        let ontology = ontology_store.get(resolved_id)?;
        let out = serde_json::json!({
            "status": "ok",
            "id": resolved_id,
            "ontology": ontology,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Updated ontology for {}", row.title);
    }

    Ok(())
}

pub fn run_search(args: OntologySearchArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let ontology_store = OntologyStore::open(&ws.data_path().join("entities.db"))?;

    let results = ontology_store.search(&args.query)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        if results.is_empty() {
            println!("No ontology entries matching '{}'.", args.query);
            return Ok(());
        }

        println!("Ontology entries matching '{}':\n", args.query);
        for entry in &results {
            println!(
                "  [{}] {} — entity: {}...",
                entry.category,
                entry.value,
                &entry.entity_id[..8]
            );
        }
        println!("\n{} results", results.len());
    }

    Ok(())
}
