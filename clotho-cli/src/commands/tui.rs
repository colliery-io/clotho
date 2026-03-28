use std::path::PathBuf;
use std::process::Command;

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

    /// Run TUI only (no tmux, no claude pane).
    #[arg(long)]
    pub no_tmux: bool,
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

    if args.no_tmux {
        // Direct TUI mode (no claude pane)
        let rt = tokio::runtime::Runtime::new()?;
        return rt.block_on(clotho_tui::run(workspace));
    }

    // Launch via tmux with two panes: TUI (top) + claude (bottom)
    launch_tmux(&workspace, &args.claude_args)
}

fn launch_tmux(workspace: &PathBuf, claude_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Check tmux is available
    if Command::new("tmux").arg("-V").output().is_err() {
        return Err("tmux is required. Install it with: brew install tmux (macOS) or apt install tmux (Linux)".into());
    }

    let session_name = "clotho";
    let ws_str = workspace.display().to_string();

    // Build the clotho TUI command
    let tui_cmd = format!("clotho tui --no-tmux -w {}", ws_str);

    // Build the claude command
    let mut claude_cmd = format!("cd {} && claude", ws_str);
    for arg in claude_args {
        claude_cmd.push(' ');
        claude_cmd.push_str(arg);
    }

    // Kill existing session if any
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", session_name])
        .output();

    // Create new session with TUI in the first pane
    let status = Command::new("tmux")
        .args([
            "new-session", "-d",
            "-s", session_name,
            "-x", "200", "-y", "50",
            &tui_cmd,
        ])
        .status()?;

    if !status.success() {
        return Err("Failed to create tmux session".into());
    }

    // Split horizontally: claude in the bottom pane
    let status = Command::new("tmux")
        .args([
            "split-window", "-v",
            "-t", session_name,
            "-p", "50",
            &claude_cmd,
        ])
        .status()?;

    if !status.success() {
        return Err("Failed to create claude pane".into());
    }

    // Select the bottom pane (claude) as active
    let _ = Command::new("tmux")
        .args(["select-pane", "-t", &format!("{}:.1", session_name)])
        .status();

    // Enable mouse support in this session
    let _ = Command::new("tmux")
        .args(["set-option", "-t", session_name, "mouse", "on"])
        .status();

    // Attach to the session
    let status = Command::new("tmux")
        .args(["attach-session", "-t", session_name])
        .status()?;

    if !status.success() {
        return Err("tmux session ended with error".into());
    }

    Ok(())
}
