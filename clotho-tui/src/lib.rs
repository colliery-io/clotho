mod app;
mod editor;
mod event;
mod navigator;
mod pty;
mod state;
mod ui;

pub use app::App;

use std::path::PathBuf;

/// Launch the Clotho TUI.
///
/// `workspace` is the path to the `.clotho/` directory.
/// `claude_args` are optional extra arguments passed to the embedded claude CLI.
pub async fn run(workspace: PathBuf, claude_args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(workspace, claude_args)?;
    app.run().await
}
