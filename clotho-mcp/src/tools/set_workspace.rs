use crate::formatting::text_result;
use crate::workspace_resolver;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_set_workspace",
    description = "Set the workspace path for the current session. All subsequent tools will use this path. The server auto-detects on startup, but use this to override or set manually.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SetWorkspaceTool {
    /// Path to the directory containing .clotho/
    pub path: String,
}

impl SetWorkspaceTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let p = Path::new(&self.path);
        if !p.join(".clotho").is_dir() {
            return Err(CallToolError::new(std::io::Error::other(format!(
                "No .clotho/ directory found at {}",
                self.path
            ))));
        }

        workspace_resolver::set_workspace(self.path.clone());

        Ok(text_result(format!(
            "## Workspace Set\n\nWorkspace path set to `{}`",
            self.path
        )))
    }
}
