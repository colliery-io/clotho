use serde::{Deserialize, Serialize};

use crate::domain::types::{EntityId, EntityType};
use crate::error::GraphError;
use crate::graph::GraphStore;

/// Metadata about a node in the graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub title: String,
}

impl GraphStore {
    /// Register an entity as a node in the graph.
    ///
    /// Upserts — if the node already exists, its properties are updated.
    pub fn register_node(
        &self,
        id: &EntityId,
        entity_type: EntityType,
        title: &str,
    ) -> Result<(), GraphError> {
        let props = vec![
            ("entity_type", entity_type.to_string()),
            ("title", title.to_string()),
        ];
        // Convert to (String, String) pairs for graphqlite
        let props: Vec<(String, String)> =
            props.into_iter().map(|(k, v)| (k.to_string(), v)).collect();

        self.graph()
            .upsert_node(&id.to_string(), props, &entity_type.to_string())
            .map_err(|e| GraphError::QueryFailed(e.to_string()))
    }

    /// Remove a node and all its edges from the graph.
    pub fn remove_node(&self, id: &EntityId) -> Result<(), GraphError> {
        self.graph()
            .delete_node(&id.to_string())
            .map_err(|e| GraphError::QueryFailed(e.to_string()))
    }

    /// Get metadata about a node by querying with Cypher.
    pub fn get_node(&self, id: &EntityId) -> Result<Option<NodeInfo>, GraphError> {
        let query = format!(
            "MATCH (n {{id: '{}'}}) RETURN n.entity_type AS entity_type, n.title AS title",
            graphqlite::escape_string(&id.to_string())
        );
        let result = self
            .graph()
            .connection()
            .cypher(&query)
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        if result.is_empty() {
            return Ok(None);
        }

        let entity_type_str: String = result[0].get("entity_type").unwrap_or_default();
        let title: String = result[0].get("title").unwrap_or_default();

        let entity_type = parse_entity_type(&entity_type_str).ok_or_else(|| {
            GraphError::QueryFailed(format!("unknown entity type: {}", entity_type_str))
        })?;

        Ok(Some(NodeInfo {
            id: id.clone(),
            entity_type,
            title,
        }))
    }

    /// Check whether a node exists in the graph.
    pub fn has_node(&self, id: &EntityId) -> Result<bool, GraphError> {
        self.graph()
            .has_node(&id.to_string())
            .map_err(|e| GraphError::QueryFailed(e.to_string()))
    }
}

/// Parse an EntityType from its Display string.
pub(crate) fn parse_entity_type(s: &str) -> Option<EntityType> {
    match s {
        "Program" => Some(EntityType::Program),
        "Responsibility" => Some(EntityType::Responsibility),
        "Objective" => Some(EntityType::Objective),
        "Workstream" => Some(EntityType::Workstream),
        "Task" => Some(EntityType::Task),
        "Meeting" => Some(EntityType::Meeting),
        "Transcript" => Some(EntityType::Transcript),
        "Note" => Some(EntityType::Note),
        "Reflection" => Some(EntityType::Reflection),
        "Artifact" => Some(EntityType::Artifact),
        "Decision" => Some(EntityType::Decision),
        "Risk" => Some(EntityType::Risk),
        "Blocker" => Some(EntityType::Blocker),
        "Question" => Some(EntityType::Question),
        "Insight" => Some(EntityType::Insight),
        "Person" => Some(EntityType::Person),
        _ => None,
    }
}
