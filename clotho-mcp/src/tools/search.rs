use crate::formatting::text_result;
use crate::workspace_resolver;
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_search",
    description = "Full-text keyword search across all entities in the Clotho workspace.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchTool {
    /// Search query (FTS5 keywords)
    pub query: String,
    /// Maximum number of results (default 10)
    pub limit: Option<u32>,
}

impl SearchTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let index = SearchIndex::open(&ws.index_path().join("search.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let mut results = index
            .search(&self.query)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let limit = self.limit.unwrap_or(10) as usize;
        results.truncate(limit);

        if results.is_empty() {
            return Ok(text_result(format!(
                "No results found for '{}'.",
                self.query
            )));
        }

        let mut output = format!(
            "## Search Results for \"{}\"\n\nFound {} results\n\n",
            self.query,
            results.len()
        );
        for (i, r) in results.iter().enumerate() {
            output.push_str(&format!(
                "{}. **[{}]** {}\n   {}\n   ID: `{}`\n\n",
                i + 1,
                r.entity_type,
                r.title,
                r.snippet,
                r.entity_id
            ));
        }

        Ok(text_result(output))
    }
}
