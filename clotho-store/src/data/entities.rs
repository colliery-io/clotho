use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::StoreError;

/// Result of resolving a potentially-partial entity ID.
#[derive(Debug)]
pub enum ResolveResult {
    /// Full UUID matched exactly.
    Exact(EntityRow),
    /// Prefix matched exactly one entity.
    Unique(EntityRow),
    /// Prefix matched multiple entities — caller must handle.
    Ambiguous(Vec<EntityRow>),
    /// Nothing matched.
    NotFound,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS entities (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    status TEXT,
    task_state TEXT,
    extraction_status TEXT,
    source_transcript_id TEXT,
    source_span_start INTEGER,
    source_span_end INTEGER,
    confidence REAL,
    content_path TEXT,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_status ON entities(status);
CREATE INDEX IF NOT EXISTS idx_entities_task_state ON entities(task_state);
CREATE INDEX IF NOT EXISTS idx_entities_extraction_status ON entities(extraction_status);
"#;

/// A flat row representing an entity in SQLite.
///
/// This is the serializable bridge between domain entity types and the database.
/// Conversion to/from domain entities happens at a higher layer (sync).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRow {
    pub id: String,
    pub entity_type: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: Option<String>,
    pub task_state: Option<String>,
    pub extraction_status: Option<String>,
    pub source_transcript_id: Option<String>,
    pub source_span_start: Option<i64>,
    pub source_span_end: Option<i64>,
    pub confidence: Option<f64>,
    pub content_path: Option<String>,
    pub metadata: Option<String>,
}

/// SQLite-backed entity storage (data/entities.db).
pub struct EntityStore {
    conn: Connection,
}

impl EntityStore {
    /// Open or create the entity store at the given path.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Open an in-memory entity store (for tests).
    pub fn in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Insert a new entity row.
    pub fn insert(&self, row: &EntityRow) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO entities (id, entity_type, title, created_at, updated_at, status, task_state, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, content_path, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                row.id,
                row.entity_type,
                row.title,
                row.created_at,
                row.updated_at,
                row.status,
                row.task_state,
                row.extraction_status,
                row.source_transcript_id,
                row.source_span_start,
                row.source_span_end,
                row.confidence,
                row.content_path,
                row.metadata,
            ],
        )?;
        Ok(())
    }

    /// Update an existing entity row.
    pub fn update(&self, row: &EntityRow) -> Result<(), StoreError> {
        let rows_changed = self.conn.execute(
            "UPDATE entities SET entity_type=?2, title=?3, created_at=?4, updated_at=?5, status=?6, task_state=?7, extraction_status=?8, source_transcript_id=?9, source_span_start=?10, source_span_end=?11, confidence=?12, content_path=?13, metadata=?14
             WHERE id=?1",
            params![
                row.id,
                row.entity_type,
                row.title,
                row.created_at,
                row.updated_at,
                row.status,
                row.task_state,
                row.extraction_status,
                row.source_transcript_id,
                row.source_span_start,
                row.source_span_end,
                row.confidence,
                row.content_path,
                row.metadata,
            ],
        )?;
        if rows_changed == 0 {
            return Err(StoreError::EntityNotFound(row.id.clone()));
        }
        Ok(())
    }

    /// Get an entity by ID.
    pub fn get(&self, id: &str) -> Result<Option<EntityRow>, StoreError> {
        let row = self
            .conn
            .query_row(
                "SELECT id, entity_type, title, created_at, updated_at, status, task_state, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, content_path, metadata FROM entities WHERE id=?1",
                params![id],
                row_to_entity_row,
            )
            .optional()?;
        Ok(row)
    }

    /// Delete an entity by ID.
    pub fn delete(&self, id: &str) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM entities WHERE id=?1", params![id])?;
        Ok(())
    }

    /// List entities by type.
    pub fn list_by_type(&self, entity_type: &str) -> Result<Vec<EntityRow>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, entity_type, title, created_at, updated_at, status, task_state, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, content_path, metadata FROM entities WHERE entity_type=?1")?;
        let rows = stmt
            .query_map(params![entity_type], row_to_entity_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// List entities by status (Active/Inactive).
    pub fn list_by_status(&self, status: &str) -> Result<Vec<EntityRow>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, entity_type, title, created_at, updated_at, status, task_state, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, content_path, metadata FROM entities WHERE status=?1")?;
        let rows = stmt
            .query_map(params![status], row_to_entity_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// List entities by task state (Todo/Doing/Blocked/Done).
    pub fn list_by_state(&self, state: &str) -> Result<Vec<EntityRow>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, entity_type, title, created_at, updated_at, status, task_state, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, content_path, metadata FROM entities WHERE task_state=?1")?;
        let rows = stmt
            .query_map(params![state], row_to_entity_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// List all entities.
    pub fn list_all(&self) -> Result<Vec<EntityRow>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, entity_type, title, created_at, updated_at, status, task_state, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, content_path, metadata FROM entities")?;
        let rows = stmt
            .query_map([], row_to_entity_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Find entities whose ID starts with the given prefix.
    pub fn get_by_prefix(&self, prefix: &str) -> Result<Vec<EntityRow>, StoreError> {
        let pattern = format!("{}%", prefix);
        let mut stmt = self.conn.prepare(
            "SELECT id, entity_type, title, created_at, updated_at, status, task_state, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, content_path, metadata FROM entities WHERE id LIKE ?1",
        )?;
        let rows = stmt
            .query_map(params![pattern], row_to_entity_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Resolve a full or partial entity ID. Tries exact match first, then prefix.
    pub fn resolve_id(&self, input: &str) -> Result<ResolveResult, StoreError> {
        // Try exact match first
        if let Some(row) = self.get(input)? {
            return Ok(ResolveResult::Exact(row));
        }

        // Try prefix match
        let matches = self.get_by_prefix(input)?;
        match matches.len() {
            0 => Ok(ResolveResult::NotFound),
            1 => Ok(ResolveResult::Unique(matches.into_iter().next().unwrap())),
            _ => Ok(ResolveResult::Ambiguous(matches)),
        }
    }

    /// Access the underlying connection (for federation ATTACH).
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

fn row_to_entity_row(row: &rusqlite::Row) -> rusqlite::Result<EntityRow> {
    Ok(EntityRow {
        id: row.get(0)?,
        entity_type: row.get(1)?,
        title: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        status: row.get(5)?,
        task_state: row.get(6)?,
        extraction_status: row.get(7)?,
        source_transcript_id: row.get(8)?,
        source_span_start: row.get(9)?,
        source_span_end: row.get(10)?,
        confidence: row.get(11)?,
        content_path: row.get(12)?,
        metadata: row.get(13)?,
    })
}
