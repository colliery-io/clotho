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
use std::collections::HashMap;
use std::path::Path;

#[mcp_tool(
    name = "clotho_workspace_summary",
    description = "Get a high-level overview of the workspace: entity counts by type, active/blocked tasks, unprocessed transcripts, and recent activity. Use this instead of multiple list_entities calls.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceSummaryTool {}

impl WorkspaceSummaryTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let all = store
            .list_all()
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        // Count by type
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        let mut task_active = 0usize;
        let mut task_blocked = 0usize;
        let mut task_todo = 0usize;
        let mut task_done = 0usize;
        let mut transcript_ids: Vec<String> = Vec::new();
        let mut recent: Vec<(String, String, String, String)> = Vec::new(); // (id, type, title, updated)

        for row in &all {
            *type_counts.entry(row.entity_type.clone()).or_insert(0) += 1;

            if row.entity_type == "Task" {
                match row.task_state.as_deref() {
                    Some("doing") => task_active += 1,
                    Some("blocked") => task_blocked += 1,
                    Some("todo") => task_todo += 1,
                    Some("done") => task_done += 1,
                    _ => {}
                }
            }

            if row.entity_type == "Transcript" {
                transcript_ids.push(row.id.clone());
            }

            recent.push((
                row.id.clone(),
                row.entity_type.clone(),
                row.title.clone(),
                row.updated_at.clone(),
            ));
        }

        // Sort recent by updated_at desc, take top 10
        recent.sort_by(|a, b| b.3.cmp(&a.3));
        recent.truncate(10);

        // Count unprocessed transcripts
        let mut unprocessed = 0usize;
        if !transcript_ids.is_empty() {
            if let Ok(log) = ProcessingLog::open(&ws.data_path().join("entities.db")) {
                for tid in &transcript_ids {
                    if let Ok(history) = log.get_history(tid) {
                        let has_extraction = history.iter().any(|r| r.process_name == "extraction");
                        if !has_extraction {
                            unprocessed += 1;
                        }
                    } else {
                        unprocessed += 1;
                    }
                }
            }
        }

        // Build output
        let mut output = format!(
            "## Workspace Summary\n\n**{} entities total**\n\n",
            all.len()
        );

        // Entity counts table
        output.push_str("### Entities by Type\n\n| Type | Count |\n|---|---|\n");
        let mut sorted_types: Vec<_> = type_counts.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));
        for (t, c) in &sorted_types {
            output.push_str(&format!("| {} | {} |\n", t, c));
        }

        // Task breakdown
        let total_tasks = task_todo + task_active + task_blocked + task_done;
        if total_tasks > 0 {
            output.push_str(&format!(
                "\n### Tasks ({} total)\n\n| State | Count |\n|---|---|\n| Todo | {} |\n| Active | {} |\n| Blocked | {} |\n| Done | {} |\n",
                total_tasks, task_todo, task_active, task_blocked, task_done
            ));
        }

        // Unprocessed transcripts
        if !transcript_ids.is_empty() {
            output.push_str(&format!(
                "\n### Transcripts\n\n{} total, **{} unprocessed**\n",
                transcript_ids.len(),
                unprocessed
            ));
        }

        // Recent activity
        if !recent.is_empty() {
            output.push_str(
                "\n### Recent Activity\n\n| ID | Type | Title | Updated |\n|---|---|---|---|\n",
            );
            for (id, et, title, updated) in &recent {
                let short_date = if updated.len() > 10 {
                    &updated[..10]
                } else {
                    updated
                };
                output.push_str(&format!(
                    "| `{}` | {} | {} | {} |\n",
                    &id[..8],
                    et,
                    title,
                    short_date
                ));
            }
        }

        Ok(text_result(output))
    }
}
