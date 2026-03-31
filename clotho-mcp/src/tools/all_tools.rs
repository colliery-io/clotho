use rust_mcp_sdk::schema::Tool;

use super::{
    ArchiveEntityTool, BatchCreateRelationsTool, CaptureDirectoryTool, CaptureTool,
    CheckProcessedTool, CreateEntityTool, CreateNoteTool, CreateReflectionTool,
    CreateRelationTool, DeleteEntityTool, DeleteRelationTool, GetOntologyTool, GetRelationsTool,
    InitTool, ListEntitiesTool, ListSurfacesTool, ListUnprocessedTool, MarkProcessedTool,
    PushSurfaceTool, QueryTool, ReadEntityTool, ReadSurfaceTool, SearchOntologyTool, SearchTool,
    SetWorkspaceTool, SyncTool, UpdateEntityTool, UpdateOntologyTool, WorkspaceSummaryTool,
};

/// Registry of all Clotho MCP tools.
pub struct ClothoTools;

impl ClothoTools {
    pub fn tools() -> Vec<Tool> {
        vec![
            // Session
            SetWorkspaceTool::tool(),
            // Read-only
            SearchTool::tool(),
            QueryTool::tool(),
            ReadEntityTool::tool(),
            ListEntitiesTool::tool(),
            GetRelationsTool::tool(),
            WorkspaceSummaryTool::tool(),
            ListUnprocessedTool::tool(),
            // Write - workspace
            InitTool::tool(),
            CaptureTool::tool(),
            CaptureDirectoryTool::tool(),
            CreateNoteTool::tool(),
            CreateReflectionTool::tool(),
            // Write - entity CRUD
            CreateEntityTool::tool(),
            UpdateEntityTool::tool(),
            DeleteEntityTool::tool(),
            ArchiveEntityTool::tool(),
            // Write - relations
            CreateRelationTool::tool(),
            BatchCreateRelationsTool::tool(),
            DeleteRelationTool::tool(),
            // Sync
            SyncTool::tool(),
            // Ontology
            GetOntologyTool::tool(),
            UpdateOntologyTool::tool(),
            SearchOntologyTool::tool(),
            // Processing log
            CheckProcessedTool::tool(),
            MarkProcessedTool::tool(),
            // Surfaces (TUI)
            PushSurfaceTool::tool(),
            ReadSurfaceTool::tool(),
            ListSurfacesTool::tool(),
        ]
    }
}
