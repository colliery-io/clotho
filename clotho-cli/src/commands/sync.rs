use clap::Args;

use clotho_store::workspace::Workspace;
use clotho_sync::SyncEngine;

#[derive(Args)]
pub struct SyncArgs {
    /// Prune history after sync (keep last 20 commits).
    #[arg(long)]
    pub prune: bool,

    /// Number of commits to keep when pruning (default 20).
    #[arg(long, default_value = "20")]
    pub keep: usize,
}

pub fn run(args: SyncArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let ws = Workspace::open(&std::env::current_dir()?)?;

    // Open or init the sync engine
    let engine = match SyncEngine::open(&ws.path) {
        Ok(e) => e,
        Err(_) => {
            // No git repo yet — init one
            SyncEngine::init(&ws.path)?
        }
    };

    let result = engine.sync()?;

    if args.prune {
        let pruned = engine.prune_history(args.keep)?;
        if !json && pruned > 0 {
            println!("Pruned {} commits (keeping {})", pruned, args.keep);
        }
    }

    if json {
        let out = serde_json::json!({
            "committed": result.committed,
            "pushed": result.pushed,
            "files_changed": result.files_changed,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        if result.committed {
            println!("Synced: {} files committed", result.files_changed);
            if result.pushed {
                println!("  Pushed to remote");
            }
        } else {
            println!("No changes to sync.");
        }
    }

    Ok(())
}
