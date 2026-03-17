use crate::formatting::text_result;
use clotho_core::domain::traits::RelationType;
use clotho_core::domain::types::EntityId;
use clotho_core::graph::GraphStore;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_create_relation",
    description = "Create a typed relation (graph edge) between two entities.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateRelationTool {
    /// Path to the directory containing .clotho/
    pub workspace_path: String,
    /// Source entity ID (UUID)
    pub source_id: String,
    /// Relation type (belongs_to, relates_to, delivers, spawned_from, extracted_from, has_decision, has_risk, blocked_by, mentions, has_cadence, has_deadline, has_schedule)
    pub relation_type: String,
    /// Target entity ID (UUID)
    pub target_id: String,
}

impl CreateRelationTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws = Workspace::open(Path::new(&self.workspace_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let source = parse_id(&self.source_id)?;
        let target = parse_id(&self.target_id)?;
        let rel = parse_relation_type(&self.relation_type)?;

        graph.add_edge(&source, &target, rel)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        Ok(text_result(format!(
            "## Relation Created\n\n`{}` -[{}]-> `{}`",
            &self.source_id[..8], self.relation_type.to_uppercase(), &self.target_id[..8]
        )))
    }
}

fn parse_id(s: &str) -> Result<EntityId, CallToolError> {
    uuid::Uuid::parse_str(s)
        .map(EntityId::from)
        .map_err(|e| CallToolError::new(std::io::Error::other(format!("invalid ID: {}", e))))
}

fn parse_relation_type(s: &str) -> Result<RelationType, CallToolError> {
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
        _ => Err(CallToolError::new(std::io::Error::other(format!("Unknown relation type: {}", s)))),
    }
}
