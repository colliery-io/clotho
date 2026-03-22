pub mod formatting;
pub mod resolve;
pub mod server;
pub mod tools;
pub mod workspace_resolver;

pub use server::ClothoServerHandler;

use anyhow::Result as AnyhowResult;
use rust_mcp_sdk::{
    mcp_server::{server_runtime, McpServerOptions},
    schema::{
        Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
        LATEST_PROTOCOL_VERSION,
    },
    McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
};
use tracing::info;

/// Run the Clotho MCP server on stdio transport.
pub async fn run() -> AnyhowResult<()> {
    // Initialize tracing to stderr (stdout is for MCP protocol)
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_max_level(tracing::Level::WARN)
        .try_init();

    info!("Starting Clotho MCP Server");

    // Try to auto-detect workspace from cwd
    if let Some(path) = workspace_resolver::detect_and_set() {
        info!("Auto-detected workspace at: {}", path);
    } else {
        info!("No workspace detected. Use clotho_set_workspace to set one.");
    }

    let server_details = InitializeResult {
        server_info: Implementation {
            name: "Clotho Workspace Management".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: Some("Clotho MCP Server".to_string()),
            description: Some(
                "MCP server for personal work and time management through Clotho workspaces"
                    .to_string(),
            ),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some(include_str!("instructions.md").to_string()),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;

    let handler = ClothoServerHandler::new().to_mcp_server_handler();

    let server = server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler,
        task_store: None,
        client_task_store: None,
    });

    info!("MCP Server starting on stdio transport");
    server
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("MCP server failed to start: {}", e))?;

    Ok(())
}
