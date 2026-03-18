use rust_mcp_sdk::schema::Tool;

use super::{
    CreateEntityTool, CreateNoteTool, CreateReflectionTool, CreateRelationTool, DeleteEntityTool,
    DeleteRelationTool, GetOntologyTool, GetRelationsTool, IngestTool, InitTool,
    ListEntitiesTool, QueryTool, ReadEntityTool, SearchOntologyTool, SearchTool, SyncTool,
    UpdateEntityTool, UpdateOntologyTool,
};

/// Registry of all Clotho MCP tools.
pub struct ClothoTools;

impl ClothoTools {
    pub fn tools() -> Vec<Tool> {
        vec![
            // Read-only
            SearchTool::tool(),
            QueryTool::tool(),
            ReadEntityTool::tool(),
            ListEntitiesTool::tool(),
            GetRelationsTool::tool(),
            // Write - workspace
            InitTool::tool(),
            IngestTool::tool(),
            CreateNoteTool::tool(),
            CreateReflectionTool::tool(),
            // Write - entity CRUD
            CreateEntityTool::tool(),
            UpdateEntityTool::tool(),
            DeleteEntityTool::tool(),
            // Write - relations
            CreateRelationTool::tool(),
            DeleteRelationTool::tool(),
            // Sync
            SyncTool::tool(),
            // Ontology
            GetOntologyTool::tool(),
            UpdateOntologyTool::tool(),
            SearchOntologyTool::tool(),
        ]
    }
}
