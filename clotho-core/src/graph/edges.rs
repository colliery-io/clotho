use serde::{Deserialize, Serialize};

use crate::domain::traits::RelationType;
use crate::domain::types::EntityId;
use crate::error::GraphError;
use crate::graph::GraphStore;

/// Metadata about an edge in the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeInfo {
    pub source_id: EntityId,
    pub target_id: EntityId,
    pub relation_type: RelationType,
}

impl GraphStore {
    /// Add a typed relation between two nodes.
    ///
    /// Upserts — if the edge already exists, it's a no-op.
    pub fn add_edge(
        &self,
        source: &EntityId,
        target: &EntityId,
        rel_type: RelationType,
    ) -> Result<(), GraphError> {
        let empty_props: Vec<(String, String)> = vec![];
        self.graph()
            .upsert_edge(
                &source.to_string(),
                &target.to_string(),
                empty_props,
                &rel_type_to_string(rel_type),
            )
            .map_err(|e| GraphError::QueryFailed(e.to_string()))
    }

    /// Add a typed relation with properties (for temporal edges, etc.).
    pub fn add_edge_with_props(
        &self,
        source: &EntityId,
        target: &EntityId,
        rel_type: RelationType,
        props: Vec<(String, String)>,
    ) -> Result<(), GraphError> {
        self.graph()
            .upsert_edge(
                &source.to_string(),
                &target.to_string(),
                props,
                &rel_type_to_string(rel_type),
            )
            .map_err(|e| GraphError::QueryFailed(e.to_string()))
    }

    /// Remove a typed relation between two nodes.
    pub fn remove_edge(
        &self,
        source: &EntityId,
        target: &EntityId,
        rel_type: RelationType,
    ) -> Result<(), GraphError> {
        self.graph()
            .delete_edge(
                &source.to_string(),
                &target.to_string(),
                Some(&rel_type_to_string(rel_type)),
            )
            .map_err(|e| GraphError::QueryFailed(e.to_string()))
    }

    /// Check whether a typed relation exists between two nodes.
    pub fn has_edge(
        &self,
        source: &EntityId,
        target: &EntityId,
        rel_type: RelationType,
    ) -> Result<bool, GraphError> {
        self.graph()
            .has_edge(
                &source.to_string(),
                &target.to_string(),
                Some(&rel_type_to_string(rel_type)),
            )
            .map_err(|e| GraphError::QueryFailed(e.to_string()))
    }

    /// Get all outgoing edges from a node.
    pub fn get_edges_from(&self, source: &EntityId) -> Result<Vec<EdgeInfo>, GraphError> {
        let query = format!(
            "MATCH (a {{id: '{}'}})-[r]->(b) RETURN a.id AS source, type(r) AS rel_type, b.id AS target",
            graphqlite::escape_string(&source.to_string())
        );
        self.query_edges(&query)
    }

    /// Get outgoing edges of a specific type from a node.
    pub fn get_edges_by_type(
        &self,
        source: &EntityId,
        rel_type: RelationType,
    ) -> Result<Vec<EdgeInfo>, GraphError> {
        let rel_str = rel_type_to_string(rel_type);
        let query = format!(
            "MATCH (a {{id: '{}'}})-[r:{}]->(b) RETURN a.id AS source, type(r) AS rel_type, b.id AS target",
            graphqlite::escape_string(&source.to_string()),
            graphqlite::sanitize_rel_type(&rel_str)
        );
        self.query_edges(&query)
    }

    /// Get all incoming edges to a node.
    pub fn get_edges_to(&self, target: &EntityId) -> Result<Vec<EdgeInfo>, GraphError> {
        let query = format!(
            "MATCH (a)-[r]->(b {{id: '{}'}}) RETURN a.id AS source, type(r) AS rel_type, b.id AS target",
            graphqlite::escape_string(&target.to_string())
        );
        self.query_edges(&query)
    }

    /// Internal helper to execute an edge query and parse results.
    fn query_edges(&self, query: &str) -> Result<Vec<EdgeInfo>, GraphError> {
        let result = self
            .graph()
            .connection()
            .cypher(query)
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let mut edges = Vec::new();
        for row in result.iter() {
            let source_str: String = row.get("source").unwrap_or_default();
            let target_str: String = row.get("target").unwrap_or_default();
            let rel_type_str: String = row.get("rel_type").unwrap_or_default();

            if let (Some(source_id), Some(target_id), Some(relation_type)) = (
                parse_entity_id(&source_str),
                parse_entity_id(&target_str),
                parse_relation_type(&rel_type_str),
            ) {
                edges.push(EdgeInfo {
                    source_id,
                    target_id,
                    relation_type,
                });
            }
        }
        Ok(edges)
    }
}

/// Convert RelationType to the string used as graphqlite relationship type.
pub(crate) fn rel_type_to_string(rel_type: RelationType) -> String {
    match rel_type {
        RelationType::BelongsTo => "BELONGS_TO".to_string(),
        RelationType::RelatesTo => "RELATES_TO".to_string(),
        RelationType::Delivers => "DELIVERS".to_string(),
        RelationType::SpawnedFrom => "SPAWNED_FROM".to_string(),
        RelationType::ExtractedFrom => "EXTRACTED_FROM".to_string(),
        RelationType::HasDecision => "HAS_DECISION".to_string(),
        RelationType::HasRisk => "HAS_RISK".to_string(),
        RelationType::BlockedBy => "BLOCKED_BY".to_string(),
        RelationType::Mentions => "MENTIONS".to_string(),
        RelationType::HasCadence => "HAS_CADENCE".to_string(),
        RelationType::HasDeadline => "HAS_DEADLINE".to_string(),
        RelationType::HasSchedule => "HAS_SCHEDULE".to_string(),
    }
}

/// Parse a RelationType from its string representation.
pub(crate) fn parse_relation_type(s: &str) -> Option<RelationType> {
    match s {
        "BELONGS_TO" => Some(RelationType::BelongsTo),
        "RELATES_TO" => Some(RelationType::RelatesTo),
        "DELIVERS" => Some(RelationType::Delivers),
        "SPAWNED_FROM" => Some(RelationType::SpawnedFrom),
        "EXTRACTED_FROM" => Some(RelationType::ExtractedFrom),
        "HAS_DECISION" => Some(RelationType::HasDecision),
        "HAS_RISK" => Some(RelationType::HasRisk),
        "BLOCKED_BY" => Some(RelationType::BlockedBy),
        "MENTIONS" => Some(RelationType::Mentions),
        "HAS_CADENCE" => Some(RelationType::HasCadence),
        "HAS_DEADLINE" => Some(RelationType::HasDeadline),
        "HAS_SCHEDULE" => Some(RelationType::HasSchedule),
        _ => None,
    }
}

/// Parse an EntityId from a UUID string.
fn parse_entity_id(s: &str) -> Option<EntityId> {
    uuid::Uuid::parse_str(s).ok().map(EntityId::from)
}
