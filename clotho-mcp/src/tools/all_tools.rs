use rust_mcp_sdk::schema::Tool;

use super::{
    CreateNoteTool, CreateReflectionTool, IngestTool, InitTool, ListEntitiesTool, QueryTool,
    ReadEntityTool, SearchTool,
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
            // Write
            InitTool::tool(),
            IngestTool::tool(),
            CreateNoteTool::tool(),
            CreateReflectionTool::tool(),
        ]
    }
}
