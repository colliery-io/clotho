use std::path::PathBuf;

use clap::Parser;

/// Launch the interactive TUI.
#[derive(Parser)]
pub struct TuiArgs {
    /// Path to the .clotho/ workspace directory.
    /// Defaults to ~/.clotho.
    #[arg(short, long)]
    pub workspace: Option<PathBuf>,

    /// Arguments to pass to the embedded claude CLI
    /// (e.g., -c for continue, -r for resume).
    #[arg(last = true)]
    pub claude_args: Vec<String>,
}

fn default_workspace() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".clotho")
}

fn ensure_workspace(workspace: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if workspace.exists() {
        clotho_store::migrations::run_migrations(
            &workspace.join("data/entities.db"),
        )?;
    } else {
        let base = workspace
            .parent()
            .ok_or("invalid workspace path")?;

        clotho_store::workspace::Workspace::init(base)?;
        eprintln!("Initialized new workspace at {}", workspace.display());
    }

    Ok(())
}

pub fn run(args: TuiArgs) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = args.workspace.unwrap_or_else(default_workspace);

    ensure_workspace(&workspace)?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(clotho_tui::run(workspace, args.claude_args))
}
