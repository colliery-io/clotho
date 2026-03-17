use crate::formatting::text_result;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_init",
    description = "Initialize a new Clotho workspace with the .clotho/ directory structure.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InitTool {
    /// Path to the directory where .clotho/ will be created
    pub path: String,
}

impl InitTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws = Workspace::init(Path::new(&self.path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        Ok(text_result(format!(
            "## Workspace Initialized\n\nCreated Clotho workspace at `{}`",
            ws.path.display()
        )))
    }
}
