use rust_mcp_sdk::schema::Tool;

use super::{
    CreateEntityTool, CreateNoteTool, CreateReflectionTool, CreateRelationTool, DeleteEntityTool,
    DeleteRelationTool, GetRelationsTool, IngestTool, InitTool, ListEntitiesTool, QueryTool,
    ReadEntityTool, SearchTool, UpdateEntityTool,
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
        ]
    }
}
