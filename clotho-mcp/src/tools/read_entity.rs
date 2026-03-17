use crate::formatting::text_result;
use clotho_store::data::entities::EntityStore;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_read_entity",
    description = "Read an entity's metadata and content by ID.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadEntityTool {
    /// Path to the directory containing .clotho/
    pub workspace_path: String,
    /// Entity ID (UUID)
    pub entity_id: String,
}

impl ReadEntityTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws = Workspace::open(Path::new(&self.workspace_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let row = store
            .get(&self.entity_id)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?
            .ok_or_else(|| {
                CallToolError::new(std::io::Error::other(format!(
                    "Entity not found: {}",
                    self.entity_id
                )))
            })?;

        let mut output = format!(
            "## {} ({})\n\n| Field | Value |\n|---|---|\n| ID | `{}` |\n| Type | {} |\n| Title | {} |\n| Created | {} |\n| Updated | {} |\n",
            row.title, row.entity_type, row.id, row.entity_type, row.title, row.created_at, row.updated_at,
        );

        if let Some(ref status) = row.status {
            output.push_str(&format!("| Status | {} |\n", status));
        }
        if let Some(ref state) = row.task_state {
            output.push_str(&format!("| State | {} |\n", state));
        }
        if let Some(ref es) = row.extraction_status {
            output.push_str(&format!("| Extraction Status | {} |\n", es));
        }
        if let Some(conf) = row.confidence {
            output.push_str(&format!("| Confidence | {:.2} |\n", conf));
        }

        // Try to read content
        if let Some(ref content_path) = row.content_path {
            let path = Path::new(content_path);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    output.push_str(&format!("\n---\n\n{}", content));
                }
            }
        }

        Ok(text_result(output))
    }
}
