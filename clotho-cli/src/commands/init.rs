use std::path::PathBuf;

use clap::Args;

use clotho_store::workspace::Workspace;

#[derive(Args)]
pub struct InitArgs {
    /// Path to initialize the workspace in (defaults to current directory).
    #[arg(long, default_value = ".")]
    pub path: PathBuf,
}

pub fn run(args: InitArgs, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = args.path.canonicalize().unwrap_or(args.path.clone());
    let ws = Workspace::init(&path)?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "path": ws.path.display().to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Initialized Clotho workspace at {}", ws.path.display());
    }

    Ok(())
}
