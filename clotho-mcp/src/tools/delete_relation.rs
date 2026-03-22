use crate::formatting::text_result;
use crate::resolve;
use crate::workspace_resolver;
use clotho_core::domain::traits::RelationType;
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
    name = "clotho_delete_relation",
    description = "Remove a typed relation (graph edge) between two entities. Accepts full UUID or prefix.",
    idempotent_hint = true,
    destructive_hint = true,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteRelationTool {
    /// Source entity ID (full UUID or prefix)
    pub source_id: String,
    /// Relation type
    pub relation_type: String,
    /// Target entity ID (full UUID or prefix)
    pub target_id: String,
}

impl DeleteRelationTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let source_row = match resolve::resolve_for_write(&entity_store, &self.source_id) {
            Ok(row) => row,
            Err(result) => return Ok(result),
        };
        let target_row = match resolve::resolve_for_write(&entity_store, &self.target_id) {
            Ok(row) => row,
            Err(result) => return Ok(result),
        };

        let source = uuid::Uuid::parse_str(&source_row.id)
            .map(EntityId::from)
            .map_err(|e| {
                CallToolError::new(std::io::Error::other(format!("invalid source ID: {}", e)))
            })?;
        let target = uuid::Uuid::parse_str(&target_row.id)
            .map(EntityId::from)
            .map_err(|e| {
                CallToolError::new(std::io::Error::other(format!("invalid target ID: {}", e)))
            })?;
        let rel = parse_rel(&self.relation_type)?;

        graph
            .remove_edge(&source, &target, rel)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        Ok(text_result(format!(
            "## Relation Removed\n\nRemoved `{}` -[{}]-> `{}`",
            &source_row.id[..8],
            self.relation_type.to_uppercase(),
            &target_row.id[..8]
        )))
    }
}

fn parse_rel(s: &str) -> Result<RelationType, CallToolError> {
    match s.to_lowercase().as_str() {
        "belongs_to" => Ok(RelationType::BelongsTo),
        "relates_to" => Ok(RelationType::RelatesTo),
        "delivers" => Ok(RelationType::Delivers),
        "spawned_from" => Ok(RelationType::SpawnedFrom),
        "extracted_from" => Ok(RelationType::ExtractedFrom),
        "has_decision" => Ok(RelationType::HasDecision),
        "has_risk" => Ok(RelationType::HasRisk),
        "blocked_by" => Ok(RelationType::BlockedBy),
        "mentions" => Ok(RelationType::Mentions),
        "has_cadence" => Ok(RelationType::HasCadence),
        "has_deadline" => Ok(RelationType::HasDeadline),
        "has_schedule" => Ok(RelationType::HasSchedule),
        _ => Err(CallToolError::new(std::io::Error::other(format!(
            "Unknown relation type: {}",
            s
        )))),
    }
}
