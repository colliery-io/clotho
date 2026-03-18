use crate::formatting::text_result;
use crate::workspace_resolver;
use clotho_store::data::processing::ProcessingLog;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_check_processed",
    description = "Check if an entity has been processed by a specific process. Returns processing history including which ontologies were used and what entities were produced.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CheckProcessedTool {
    /// Entity ID to check
    pub entity_id: String,
    /// Process name to filter by (optional — returns all if omitted)
    pub process_name: Option<String>,
}

impl CheckProcessedTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let log = ProcessingLog::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let history = log.get_history(&self.entity_id)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let filtered: Vec<_> = if let Some(ref name) = self.process_name {
            history.into_iter().filter(|r| r.process_name == *name).collect()
        } else {
            history
        };

        if filtered.is_empty() {
            return Ok(text_result(format!("Entity `{}` has not been processed.", &self.entity_id[..8])));
        }

        let mut output = format!("## Processing History: `{}`\n\n| Process | Ontologies | By | At |\n|---|---|---|---|\n", &self.entity_id[..8]);
        for r in &filtered {
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                r.process_name,
                r.ontology_ids.as_deref().unwrap_or("-"),
                r.processed_by.as_deref().unwrap_or("-"),
                r.processed_at,
            ));
        }

        Ok(text_result(output))
    }
}

#[mcp_tool(
    name = "clotho_mark_processed",
    description = "Record that a process was run against an entity. Idempotent — duplicate records are silently ignored. Used by agents after extraction to prevent reprocessing.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MarkProcessedTool {
    /// Entity ID that was processed
    pub entity_id: String,
    /// Process name (e.g., "extraction", "summarization")
    pub process_name: String,
    /// Ontology IDs used (comma-separated, optional)
    pub ontology_ids: Option<String>,
    /// Who ran the process (e.g., "debrief-processor", "user")
    pub processed_by: Option<String>,
    /// Entity IDs created as output (comma-separated, optional)
    pub output_entity_ids: Option<String>,
    /// Freeform notes
    pub notes: Option<String>,
}

impl MarkProcessedTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let log = ProcessingLog::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let inserted = log.record(
            &self.entity_id,
            &self.process_name,
            self.ontology_ids.as_deref(),
            self.processed_by.as_deref(),
            self.output_entity_ids.as_deref(),
            self.notes.as_deref(),
        ).map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        if inserted {
            Ok(text_result(format!("## Recorded\n\nMarked `{}` as processed by '{}'", &self.entity_id[..8], self.process_name)))
        } else {
            Ok(text_result(format!("## Already Processed\n\n`{}` was already processed by '{}' (skipped)", &self.entity_id[..8], self.process_name)))
        }
    }
}
