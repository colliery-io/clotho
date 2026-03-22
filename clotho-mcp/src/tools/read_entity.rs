use crate::formatting::text_result;
use crate::resolve;
use crate::workspace_resolver;
use clotho_core::domain::types::EntityId;
use clotho_core::graph::GraphStore;
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
    description = "Read an entity's metadata and content by ID (full UUID or prefix). Set include_relations to also fetch graph edges.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadEntityTool {
    /// Entity ID (full UUID or prefix)
    pub entity_id: String,
    /// Include outgoing and incoming relations in the response
    pub include_relations: Option<bool>,
}

impl ReadEntityTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let row = match resolve::resolve_for_read(&store, &self.entity_id) {
            Ok(row) => row,
            Err(result) => return Ok(result),
        };

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

        // Surface metadata fields (email, parent_id, etc.)
        if let Some(ref meta_str) = row.metadata {
            if let Ok(meta) =
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(meta_str)
            {
                if let Some(serde_json::Value::String(email)) = meta.get("email") {
                    output.push_str(&format!("| Email | {} |\n", email));
                }
                if let Some(serde_json::Value::String(url)) = meta.get("url") {
                    output.push_str(&format!("| URL | {} |\n", url));
                }
            }
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

        // Include relations if requested
        if self.include_relations.unwrap_or(false) {
            if let Ok(graph) = GraphStore::open(&ws.graph_path().join("relations.db")) {
                if let Ok(eid) = uuid::Uuid::parse_str(&row.id).map(EntityId::from) {
                    let outgoing = graph.get_edges_from(&eid).unwrap_or_default();
                    let incoming = graph.get_edges_to(&eid).unwrap_or_default();

                    if !outgoing.is_empty() || !incoming.is_empty() {
                        output.push_str("\n---\n\n### Relations\n\n");
                    }

                    if !outgoing.is_empty() {
                        output.push_str("**Outgoing:**\n\n| Relation | Target |\n|---|---|\n");
                        for e in &outgoing {
                            output.push_str(&format!(
                                "| {:?} | `{}` |\n",
                                e.relation_type, e.target_id
                            ));
                        }
                        output.push('\n');
                    }

                    if !incoming.is_empty() {
                        output.push_str("**Incoming:**\n\n| Source | Relation |\n|---|---|\n");
                        for e in &incoming {
                            output.push_str(&format!(
                                "| `{}` | {:?} |\n",
                                e.source_id, e.relation_type
                            ));
                        }
                    }
                }
            }
        }

        Ok(text_result(output))
    }
}
