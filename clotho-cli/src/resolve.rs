use clotho_store::data::entities::{EntityRow, EntityStore, ResolveResult};

/// Resolve an entity ID (full or prefix) for read-only operations.
/// On ambiguous, prints all matches and returns an error message.
pub fn resolve_for_read(
    store: &EntityStore,
    input: &str,
) -> Result<EntityRow, Box<dyn std::error::Error>> {
    match store.resolve_id(input)? {
        ResolveResult::Exact(row) | ResolveResult::Unique(row) => Ok(row),
        ResolveResult::Ambiguous(rows) => Err(format_ambiguous(input, &rows).into()),
        ResolveResult::NotFound => Err(format!("Entity not found: {}", input).into()),
    }
}

/// Resolve an entity ID for destructive operations.
/// Same as read, but the error message explicitly says it refuses to proceed.
pub fn resolve_for_write(
    store: &EntityStore,
    input: &str,
) -> Result<EntityRow, Box<dyn std::error::Error>> {
    match store.resolve_id(input)? {
        ResolveResult::Exact(row) | ResolveResult::Unique(row) => Ok(row),
        ResolveResult::Ambiguous(rows) => {
            let mut msg = format_ambiguous(input, &rows);
            msg.push_str("\nRefusing to proceed — use a longer prefix to narrow down.");
            Err(msg.into())
        }
        ResolveResult::NotFound => Err(format!("Entity not found: {}", input).into()),
    }
}

fn format_ambiguous(input: &str, rows: &[EntityRow]) -> String {
    let mut msg = format!(
        "Ambiguous ID '{}' matches {} entities:\n",
        input,
        rows.len()
    );
    for row in rows {
        let short_id = if row.id.len() > 12 {
            &row.id[..12]
        } else {
            &row.id
        };
        msg.push_str(&format!(
            "  {}...  {:16} \"{}\"\n",
            short_id, row.entity_type, row.title
        ));
    }
    msg.push_str("Use a longer prefix to narrow down.");
    msg
}
