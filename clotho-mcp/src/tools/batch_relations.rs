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

/// A single relation spec within a batch.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RelationSpec {
    /// Source entity ID (full UUID or prefix)
    pub source_id: String,
    /// Relation type (belongs_to, relates_to, delivers, spawned_from, extracted_from, has_decision, has_risk, blocked_by, mentions, has_cadence, has_deadline, has_schedule)
    pub relation_type: String,
    /// Target entity ID (full UUID or prefix)
    pub target_id: String,
}

#[mcp_tool(
    name = "clotho_batch_create_relations",
    description = "Create multiple typed relations (graph edges) in a single call. Accepts an array of {source_id, relation_type, target_id} specs. All inputs are validated before any relations are created. Accepts full UUIDs or prefixes.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BatchCreateRelationsTool {
    /// Array of relations to create
    pub relations: Vec<RelationSpec>,
}

impl BatchCreateRelationsTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        if self.relations.is_empty() {
            return Ok(text_result("No relations provided."));
        }

        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        // Phase 1: Validate all inputs before creating anything
        let mut resolved: Vec<(EntityId, RelationType, EntityId, String, String)> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        for (i, spec) in self.relations.iter().enumerate() {
            let source_row = match resolve::resolve_for_write(&entity_store, &spec.source_id) {
                Ok(row) => row,
                Err(_) => {
                    errors.push(format!(
                        "[{}] source `{}`: not found or ambiguous",
                        i + 1,
                        spec.source_id
                    ));
                    continue;
                }
            };
            let target_row = match resolve::resolve_for_write(&entity_store, &spec.target_id) {
                Ok(row) => row,
                Err(_) => {
                    errors.push(format!(
                        "[{}] target `{}`: not found or ambiguous",
                        i + 1,
                        spec.target_id
                    ));
                    continue;
                }
            };

            let rel = match parse_relation_type(&spec.relation_type) {
                Ok(r) => r,
                Err(_) => {
                    errors.push(format!(
                        "[{}] unknown relation type: {}",
                        i + 1,
                        spec.relation_type
                    ));
                    continue;
                }
            };

            let source_id = match parse_id(&source_row.id) {
                Ok(id) => id,
                Err(_) => {
                    errors.push(format!(
                        "[{}] invalid source UUID: {}",
                        i + 1,
                        source_row.id
                    ));
                    continue;
                }
            };
            let target_id = match parse_id(&target_row.id) {
                Ok(id) => id,
                Err(_) => {
                    errors.push(format!(
                        "[{}] invalid target UUID: {}",
                        i + 1,
                        target_row.id
                    ));
                    continue;
                }
            };

            resolved.push((
                source_id,
                rel,
                target_id,
                source_row.id.clone(),
                target_row.id.clone(),
            ));
        }

        if !errors.is_empty() {
            let mut msg = format!(
                "## Validation Failed\n\n{} of {} relations have errors:\n\n",
                errors.len(),
                self.relations.len()
            );
            for e in &errors {
                msg.push_str(&format!("- {}\n", e));
            }
            msg.push_str("\nNo relations were created. Fix errors and retry.");
            return Ok(text_result(msg));
        }

        // Phase 2: Create all relations
        let mut created = 0;
        for (source_id, rel, target_id, source_str, target_str) in &resolved {
            match graph.add_edge(source_id, target_id, *rel) {
                Ok(_) => created += 1,
                Err(e) => {
                    errors.push(format!(
                        "`{}` -[{:?}]-> `{}`: {}",
                        &source_str[..8],
                        rel,
                        &target_str[..8],
                        e
                    ));
                }
            }
        }

        // Build summary
        let mut output = format!("## Batch Relations Created\n\n{} of {} relations created.\n\n| # | Source | Relation | Target |\n|---|---|---|---|\n", created, self.relations.len());

        for (i, (_, rel, _, source_str, target_str)) in resolved.iter().enumerate() {
            output.push_str(&format!(
                "| {} | `{}` | {:?} | `{}` |\n",
                i + 1,
                &source_str[..8],
                rel,
                &target_str[..8]
            ));
        }

        if !errors.is_empty() {
            output.push_str("\n**Errors:**\n");
            for e in &errors {
                output.push_str(&format!("- {}\n", e));
            }
        }

        Ok(text_result(output))
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
        _ => Err(CallToolError::new(std::io::Error::other(format!(
            "Unknown relation type: {}",
            s
        )))),
    }
}
