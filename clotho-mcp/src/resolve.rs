use clotho_store::data::entities::{EntityRow, EntityStore, ResolveResult};
use rust_mcp_sdk::schema::{CallToolResult, TextContent};

/// Resolve an entity ID (full or prefix) for MCP read-only tools.
/// Returns the resolved row or a structured ambiguity result.
pub fn resolve_for_read(store: &EntityStore, input: &str) -> Result<EntityRow, CallToolResult> {
    match store.resolve_id(input) {
        Ok(ResolveResult::Exact(row)) | Ok(ResolveResult::Unique(row)) => Ok(row),
        Ok(ResolveResult::Ambiguous(rows)) => Err(ambiguity_result(input, &rows)),
        Ok(ResolveResult::NotFound) => Err(not_found_result(input)),
        Err(e) => Err(error_result(&format!("Store error: {}", e))),
    }
}

/// Resolve an entity ID for MCP destructive tools.
/// Same behavior — ambiguity is surfaced, never swallowed.
pub fn resolve_for_write(store: &EntityStore, input: &str) -> Result<EntityRow, CallToolResult> {
    match store.resolve_id(input) {
        Ok(ResolveResult::Exact(row)) | Ok(ResolveResult::Unique(row)) => Ok(row),
        Ok(ResolveResult::Ambiguous(rows)) => Err(ambiguity_result_destructive(input, &rows)),
        Ok(ResolveResult::NotFound) => Err(not_found_result(input)),
        Err(e) => Err(error_result(&format!("Store error: {}", e))),
    }
}

fn format_ambiguous_table(input: &str, rows: &[EntityRow]) -> String {
    let mut text = format!(
        "## Ambiguous ID\n\nPrefix `{}` matches {} entities:\n\n| ID | Type | Title |\n|---|---|---|\n",
        input,
        rows.len()
    );
    for row in rows {
        let short_id = if row.id.len() > 12 {
            &row.id[..12]
        } else {
            &row.id
        };
        text.push_str(&format!(
            "| `{}...` | {} | {} |\n",
            short_id, row.entity_type, row.title
        ));
    }
    text
}

fn ambiguity_result(input: &str, rows: &[EntityRow]) -> CallToolResult {
    let mut text = format_ambiguous_table(input, rows);
    text.push_str("\nUse a longer prefix to narrow down.");

    CallToolResult {
        content: vec![TextContent::new(text, None, None).into()],
        is_error: Some(false),
        meta: None,
        structured_content: None,
    }
}

fn ambiguity_result_destructive(input: &str, rows: &[EntityRow]) -> CallToolResult {
    let mut text = format_ambiguous_table(input, rows);
    text.push_str("\n**Operation refused.** Use a longer prefix or the full UUID.");

    CallToolResult {
        content: vec![TextContent::new(text, None, None).into()],
        is_error: Some(false),
        meta: None,
        structured_content: None,
    }
}

fn not_found_result(input: &str) -> CallToolResult {
    CallToolResult {
        content: vec![
            TextContent::new(format!("Entity not found: `{}`", input), None, None).into(),
        ],
        is_error: Some(true),
        meta: None,
        structured_content: None,
    }
}

fn error_result(msg: &str) -> CallToolResult {
    CallToolResult {
        content: vec![TextContent::new(msg.to_string(), None, None).into()],
        is_error: Some(true),
        meta: None,
        structured_content: None,
    }
}
