use crate::formatting::text_result;
use crate::workspace_resolver;
use clotho_store::data::entities::EntityStore;
use clotho_store::data::processing::ProcessingLog;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_list_unprocessed",
    description = "List transcripts and notes that have not yet been extracted. Returns a chronologically ordered queue for the extraction pipeline.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListUnprocessedTool {
    /// Filter by entity type: transcript, note, or both (default: both)
    pub entity_type: Option<String>,
}

impl ListUnprocessedTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let log = ProcessingLog::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        // Collect candidate entities
        let filter_type = self.entity_type.as_deref().map(|s| s.to_lowercase());
        let types_to_check: Vec<&str> = match filter_type.as_deref() {
            Some("transcript") => vec!["Transcript"],
            Some("note") => vec!["Note"],
            _ => vec!["Transcript", "Note"],
        };

        let mut candidates = Vec::new();
        for t in &types_to_check {
            let entities = store
                .list_by_type(t)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
            candidates.extend(entities);
        }

        // Filter to unprocessed
        let mut unprocessed = Vec::new();
        for entity in &candidates {
            let history = log
                .get_history(&entity.id)
                .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
            let has_extraction = history.iter().any(|r| r.process_name == "extraction");
            if !has_extraction {
                unprocessed.push(entity);
            }
        }

        // Sort by created_at ascending (oldest first = process queue order)
        unprocessed.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        if unprocessed.is_empty() {
            return Ok(text_result(
                "All transcripts and notes have been processed. Nothing in the extraction queue.",
            ));
        }

        let mut output = format!(
            "## Extraction Queue\n\n**{} unprocessed** items:\n\n| # | ID | Type | Title | Captured |\n|---|---|---|---|---|\n",
            unprocessed.len()
        );

        for (i, entity) in unprocessed.iter().enumerate() {
            let short_date = if entity.created_at.len() > 10 {
                &entity.created_at[..10]
            } else {
                &entity.created_at
            };
            output.push_str(&format!(
                "| {} | `{}` | {} | {} | {} |\n",
                i + 1,
                &entity.id[..8],
                entity.entity_type,
                entity.title,
                short_date
            ));
        }

        Ok(text_result(output))
    }
}
