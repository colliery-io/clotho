use graphqlite::CypherResult;

use crate::domain::traits::RelationType;
use crate::domain::types::{EntityId, EntityType};
use crate::error::GraphError;
use crate::graph::edges::rel_type_to_string;
use crate::graph::nodes::{parse_entity_type, NodeInfo};
use crate::graph::GraphStore;

/// Summary statistics for the graph.
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
}

impl GraphStore {
    /// Get all nodes connected to a given node (regardless of direction or type).
    pub fn get_neighbors(&self, id: &EntityId) -> Result<Vec<NodeInfo>, GraphError> {
        let esc_id = graphqlite::escape_string(&id.to_string());
        // graphqlite doesn't support undirected match `--`, so query both directions
        let query = format!(
            "MATCH (a {{id: '{}'}})-[]->(b) RETURN DISTINCT b.id AS id, b.entity_type AS entity_type, b.title AS title \
             UNION \
             MATCH (b)-[]->(a {{id: '{}'}}) RETURN DISTINCT b.id AS id, b.entity_type AS entity_type, b.title AS title",
            esc_id, esc_id
        );
        self.query_nodes(&query)
    }

    /// Get nodes reached by outgoing edges of a specific type.
    pub fn get_related_by_type(
        &self,
        id: &EntityId,
        rel_type: RelationType,
    ) -> Result<Vec<NodeInfo>, GraphError> {
        let rel_str = rel_type_to_string(rel_type);
        let query = format!(
            "MATCH (a {{id: '{}'}})-[:{}]->(b) RETURN b.id AS id, b.entity_type AS entity_type, b.title AS title",
            graphqlite::escape_string(&id.to_string()),
            graphqlite::sanitize_rel_type(&rel_str)
        );
        self.query_nodes(&query)
    }

    /// Get nodes with incoming edges of a specific type pointing to this node.
    pub fn get_incoming_by_type(
        &self,
        id: &EntityId,
        rel_type: RelationType,
    ) -> Result<Vec<NodeInfo>, GraphError> {
        let rel_str = rel_type_to_string(rel_type);
        let query = format!(
            "MATCH (a)-[:{}]->(b {{id: '{}'}}) RETURN a.id AS id, a.entity_type AS entity_type, a.title AS title",
            graphqlite::sanitize_rel_type(&rel_str),
            graphqlite::escape_string(&id.to_string())
        );
        self.query_nodes(&query)
    }

    /// Get all nodes of a specific entity type.
    pub fn get_entities_by_label(
        &self,
        entity_type: EntityType,
    ) -> Result<Vec<NodeInfo>, GraphError> {
        let query = format!(
            "MATCH (n:{}) RETURN n.id AS id, n.entity_type AS entity_type, n.title AS title",
            entity_type
        );
        self.query_nodes(&query)
    }

    /// Execute a raw Cypher query. Escape hatch for custom queries from CLI/MCP.
    pub fn raw_cypher(&self, query: &str) -> Result<CypherResult, GraphError> {
        self.graph()
            .connection()
            .cypher(query)
            .map_err(|e| GraphError::QueryFailed(e.to_string()))
    }

    /// Get node and edge counts.
    pub fn stats(&self) -> Result<GraphStats, GraphError> {
        let gs = self
            .graph()
            .stats()
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;
        Ok(GraphStats {
            node_count: gs.nodes as usize,
            edge_count: gs.edges as usize,
        })
    }

    /// Internal helper to execute a node query and parse results.
    fn query_nodes(&self, query: &str) -> Result<Vec<NodeInfo>, GraphError> {
        let result = self
            .graph()
            .connection()
            .cypher(query)
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let mut nodes = Vec::new();
        for row in result.iter() {
            let id_str: String = row.get("id").unwrap_or_default();
            let entity_type_str: String = row.get("entity_type").unwrap_or_default();
            let title: String = row.get("title").unwrap_or_default();

            if let (Some(entity_id), Some(entity_type)) = (
                uuid::Uuid::parse_str(&id_str).ok().map(EntityId::from),
                parse_entity_type(&entity_type_str),
            ) {
                nodes.push(NodeInfo {
                    id: entity_id,
                    entity_type,
                    title,
                });
            }
        }
        Ok(nodes)
    }
}
