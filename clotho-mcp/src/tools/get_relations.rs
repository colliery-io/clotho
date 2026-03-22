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
    name = "clotho_get_relations",
    description = "List all relations (outgoing and incoming graph edges) for an entity. Accepts full UUID or prefix.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetRelationsTool {
    /// Entity ID (full UUID or prefix)
    pub entity_id: String,
}

impl GetRelationsTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let row = match resolve::resolve_for_read(&entity_store, &self.entity_id) {
            Ok(row) => row,
            Err(result) => return Ok(result),
        };

        let id = uuid::Uuid::parse_str(&row.id)
            .map(EntityId::from)
            .map_err(|e| CallToolError::new(std::io::Error::other(format!("invalid ID: {}", e))))?;

        let outgoing = graph
            .get_edges_from(&id)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let incoming = graph
            .get_edges_to(&id)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        if outgoing.is_empty() && incoming.is_empty() {
            return Ok(text_result(format!("No relations for entity `{}`", row.id)));
        }

        let mut output = format!("## Relations for `{}`\n\n", &row.id[..8]);

        if !outgoing.is_empty() {
            output.push_str("### Outgoing\n\n| Relation | Target |\n|---|---|\n");
            for e in &outgoing {
                output.push_str(&format!("| {:?} | `{}` |\n", e.relation_type, e.target_id));
            }
            output.push('\n');
        }

        if !incoming.is_empty() {
            output.push_str("### Incoming\n\n| Source | Relation |\n|---|---|\n");
            for e in &incoming {
                output.push_str(&format!("| `{}` | {:?} |\n", e.source_id, e.relation_type));
            }
        }

        output.push_str(&format!(
            "\n{} outgoing, {} incoming",
            outgoing.len(),
            incoming.len()
        ));

        Ok(text_result(output))
    }
}
