use rust_mcp_sdk::schema::{CallToolResult, TextContent};

/// Helper to build a CallToolResult from text content.
pub fn text_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult {
        content: vec![TextContent::new(text.into(), None, None).into()],
        is_error: None,
        meta: None,
        structured_content: None,
    }
}

/// Helper to build an error CallToolResult.
pub fn error_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult {
        content: vec![TextContent::new(text.into(), None, None).into()],
        is_error: Some(true),
        meta: None,
        structured_content: None,
    }
}
