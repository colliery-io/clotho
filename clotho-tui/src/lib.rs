mod app;
mod editor;
mod event;
mod navigator;
mod state;
mod ui;

pub use app::App;

use std::path::PathBuf;

/// Launch the Clotho TUI (dashboard only — no embedded chat).
///
/// `workspace` is the path to the `.clotho/` directory.
pub async fn run(workspace: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(workspace)?;
    app.run().await
}
