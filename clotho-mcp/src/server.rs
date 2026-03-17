use crate::tools::{
    ClothoTools, CreateEntityTool, CreateNoteTool, CreateReflectionTool, CreateRelationTool,
    DeleteEntityTool, DeleteRelationTool, GetRelationsTool, IngestTool, InitTool,
    ListEntitiesTool, QueryTool, ReadEntityTool, SearchTool, SyncTool, UpdateEntityTool,
};
use async_trait::async_trait;
use rust_mcp_sdk::{
    mcp_server::ServerHandler,
    schema::{
        CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams, RpcError,
    },
    McpServer,
};
use std::sync::Arc;
use tracing::info;

pub struct ClothoServerHandler;

impl ClothoServerHandler {
    pub fn new() -> Self {
        info!("Initializing Clotho MCP Server");
        Self
    }
}

#[async_trait]
impl ServerHandler for ClothoServerHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: ClothoTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, rust_mcp_sdk::schema::schema_utils::CallToolError> {
        let args = serde_json::Value::Object(params.arguments.unwrap_or_default());

        match params.name.as_str() {
            "clotho_search" => {
                let tool: SearchTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_query" => {
                let tool: QueryTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_read_entity" => {
                let tool: ReadEntityTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_list_entities" => {
                let tool: ListEntitiesTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_init" => {
                let tool: InitTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_ingest" => {
                let tool: IngestTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_create_note" => {
                let tool: CreateNoteTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_create_reflection" => {
                let tool: CreateReflectionTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_create_entity" => {
                let tool: CreateEntityTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_update_entity" => {
                let tool: UpdateEntityTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_delete_entity" => {
                let tool: DeleteEntityTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_create_relation" => {
                let tool: CreateRelationTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_delete_relation" => {
                let tool: DeleteRelationTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_get_relations" => {
                let tool: GetRelationsTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "clotho_sync" => {
                let tool: SyncTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            _ => Err(
                rust_mcp_sdk::schema::schema_utils::CallToolError::unknown_tool(params.name),
            ),
        }
    }
}
