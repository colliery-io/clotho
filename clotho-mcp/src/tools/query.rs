use crate::formatting::text_result;
use crate::workspace_resolver;
use clotho_core::graph::GraphStore;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_query",
    description = "Run a raw Cypher query against the Clotho relation graph.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryTool {
    /// Cypher query to execute
    pub cypher: String,
}

impl QueryTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let result = graph
            .raw_cypher(&self.cypher)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        if result.is_empty() {
            return Ok(text_result("No results."));
        }

        let columns = result.columns().to_vec();
        let mut output = format!(
            "## Query Results\n\n| {} |\n|{}|\n",
            columns.join(" | "),
            columns.iter().map(|_| "---").collect::<Vec<_>>().join("|"),
        );

        for row in result.iter() {
            let vals: Vec<String> = columns
                .iter()
                .map(|col| {
                    let val: String = row.get(col).unwrap_or_default();
                    val
                })
                .collect();
            output.push_str(&format!("| {} |\n", vals.join(" | ")));
        }

        output.push_str(&format!("\n{} rows", result.len()));

        Ok(text_result(output))
    }
}
