use clap::Args;

use clotho_store::data::processing::ProcessingLog;
use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct ProcessedCheckArgs {
    /// Entity ID to check.
    pub id: String,

    /// Process name to check for (e.g., "extraction").
    #[arg(long)]
    pub process: Option<String>,
}

#[derive(Args)]
pub struct ProcessedMarkArgs {
    /// Entity ID to mark as processed.
    pub id: String,

    /// Process name (e.g., "extraction").
    #[arg(long)]
    pub process: String,

    /// Ontology IDs used (comma-separated).
    #[arg(long)]
    pub ontology_ids: Option<String>,

    /// Who ran the process.
    #[arg(long)]
    pub by: Option<String>,

    /// Entity IDs created as output (comma-separated).
    #[arg(long)]
    pub output_ids: Option<String>,

    /// Freeform notes.
    #[arg(long)]
    pub notes: Option<String>,
}

pub fn run_check(args: ProcessedCheckArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let log = ProcessingLog::open(&ws.data_path().join("entities.db"))?;

    let history = log.get_history(&args.id)?;

    let filtered: Vec<_> = if let Some(ref process) = args.process {
        history.into_iter().filter(|r| r.process_name == *process).collect()
    } else {
        history
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
    } else {
        if filtered.is_empty() {
            println!("No processing records for entity {}.", &args.id[..8]);
            return Ok(());
        }

        println!("Processing history for {}:\n", &args.id[..8]);
        for record in &filtered {
            println!("  [{}] {} — by: {} at {}",
                record.process_name,
                record.ontology_ids.as_deref().unwrap_or("(no ontology)"),
                record.processed_by.as_deref().unwrap_or("unknown"),
                record.processed_at,
            );
            if let Some(ref output) = record.output_entity_ids {
                println!("    Output: {}", output);
            }
            if let Some(ref notes) = record.notes {
                println!("    Notes: {}", notes);
            }
        }
    }

    Ok(())
}

pub fn run_mark(args: ProcessedMarkArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;
    let log = ProcessingLog::open(&ws.data_path().join("entities.db"))?;

    let inserted = log.record(
        &args.id,
        &args.process,
        args.ontology_ids.as_deref(),
        args.by.as_deref(),
        args.output_ids.as_deref(),
        args.notes.as_deref(),
    )?;

    if json {
        let out = serde_json::json!({
            "status": if inserted { "recorded" } else { "already_processed" },
            "entity_id": args.id,
            "process": args.process,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        if inserted {
            println!("Marked {} as processed by '{}'", &args.id[..8], args.process);
        } else {
            println!("Already processed: {} by '{}' (skipped)", &args.id[..8], args.process);
        }
    }

    Ok(())
}
