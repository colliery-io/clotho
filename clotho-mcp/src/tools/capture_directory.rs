use crate::formatting::text_result;
use crate::workspace_resolver;
use chrono::Utc;
use clotho_core::domain::types::{EntityId, EntityType};
use clotho_core::graph::GraphStore;
use clotho_store::content::ContentStore;
use clotho_store::data::entities::{EntityRow, EntityStore};
use clotho_store::data::jsonl::{Event, EventStore, EventType};
use clotho_store::index::SearchIndex;
use clotho_store::workspace::Workspace;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "clotho_capture_directory",
    description = "Capture all matching files from a directory into the workspace. Supports glob patterns (e.g., '*.md'). Returns a summary of all captured entities.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaptureDirectoryTool {
    /// Directory path to scan
    pub path: String,
    /// Glob pattern to match files (e.g., "*.md", "*.txt"). Default: "*.md"
    pub pattern: Option<String>,
    /// Entity type for all captured files: note, meeting, transcript, artifact (default: note)
    pub entity_type: Option<String>,
}

impl CaptureDirectoryTool {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ws_path = workspace_resolver::require_workspace()
            .map_err(|e| CallToolError::new(std::io::Error::other(e)))?;
        let ws = Workspace::open(Path::new(&ws_path))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        let dir = Path::new(&self.path);
        if !dir.is_dir() {
            return Err(CallToolError::new(std::io::Error::other(format!(
                "Not a directory: {}",
                self.path
            ))));
        }

        let et = parse_entity_type(self.entity_type.as_deref().unwrap_or("note"))?;
        let pattern = self.pattern.as_deref().unwrap_or("*.md");

        // Collect matching files
        let mut files: Vec<std::path::PathBuf> = Vec::new();
        let entries = std::fs::read_dir(dir)
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
            let path = entry.path();
            if path.is_file() && matches_glob(&path, pattern) {
                files.push(path);
            }
        }

        files.sort();

        if files.is_empty() {
            return Ok(text_result(format!(
                "No files matching '{}' found in `{}`",
                pattern, self.path
            )));
        }

        let content_store = ContentStore::new(&ws.project_root());
        let entity_store = EntityStore::open(&ws.data_path().join("entities.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let graph = GraphStore::open(&ws.graph_path().join("relations.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let index = SearchIndex::open(&ws.index_path().join("search.db"))
            .map_err(|e| CallToolError::new(std::io::Error::other(e.to_string())))?;
        let events = EventStore::new(&ws.data_path());

        let mut captured = 0;
        let mut skipped = 0;
        let mut errors: Vec<String> = Vec::new();
        let mut summary_rows: Vec<String> = Vec::new();

        for file in &files {
            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!(
                        "{}: {}",
                        file.file_name().unwrap_or_default().to_string_lossy(),
                        e
                    ));
                    skipped += 1;
                    continue;
                }
            };

            let title = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string();

            let id = EntityId::new();
            let now = Utc::now();

            let content_path = match content_store.write_content(et, &id, &content) {
                Ok(p) => p,
                Err(e) => {
                    errors.push(format!("{}: {}", title, e));
                    skipped += 1;
                    continue;
                }
            };

            let row = EntityRow {
                id: id.to_string(),
                entity_type: format!("{}", et),
                title: title.clone(),
                created_at: now.to_rfc3339(),
                updated_at: now.to_rfc3339(),
                status: Some("active".to_string()),
                task_state: None,
                extraction_status: None,
                source_transcript_id: None,
                source_span_start: None,
                source_span_end: None,
                confidence: None,
                content_path: Some(content_path.display().to_string()),
                metadata: None,
            };

            if let Err(e) = entity_store.insert(&row) {
                errors.push(format!("{}: {}", title, e));
                skipped += 1;
                continue;
            }

            let _ = graph.register_node(&id, et, &title);
            let _ = index.index_entity(&id.to_string(), &format!("{}", et), &title, &content);
            let _ = events.log(&Event {
                timestamp: now,
                event_type: EventType::Created,
                entity_id: id.to_string(),
                details: Some(serde_json::json!({"source_file": file.display().to_string()})),
            });

            summary_rows.push(format!("| `{}` | {} |", &id.to_string()[..8], title));
            captured += 1;
        }

        let mut output = format!(
            "## Directory Captured\n\n**{}** files captured, **{}** skipped from `{}`\n\n",
            captured, skipped, self.path
        );

        if !summary_rows.is_empty() {
            output.push_str("| ID | Title |\n|---|---|\n");
            for row in &summary_rows {
                output.push_str(row);
                output.push('\n');
            }
        }

        if !errors.is_empty() {
            output.push_str("\n**Errors:**\n");
            for e in &errors {
                output.push_str(&format!("- {}\n", e));
            }
        }

        Ok(text_result(output))
    }
}

fn matches_glob(path: &Path, pattern: &str) -> bool {
    let filename = match path.file_name().and_then(|s| s.to_str()) {
        Some(f) => f,
        None => return false,
    };

    // Simple glob: *.ext
    if let Some(ext) = pattern.strip_prefix("*.") {
        return filename.ends_with(&format!(".{}", ext));
    }

    // Exact match fallback
    filename == pattern
}

fn parse_entity_type(s: &str) -> Result<EntityType, CallToolError> {
    match s.to_lowercase().as_str() {
        "note" => Ok(EntityType::Note),
        "meeting" => Ok(EntityType::Meeting),
        "transcript" => Ok(EntityType::Transcript),
        "artifact" => Ok(EntityType::Artifact),
        _ => Err(CallToolError::new(std::io::Error::other(format!(
            "Unknown type '{}'. Valid: note, meeting, transcript, artifact",
            s
        )))),
    }
}
