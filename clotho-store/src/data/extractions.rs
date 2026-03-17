use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::StoreError;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS extractions (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    title TEXT NOT NULL,
    speech_act TEXT,
    extraction_status TEXT NOT NULL DEFAULT 'draft',
    source_transcript_id TEXT,
    source_span_start INTEGER,
    source_span_end INTEGER,
    confidence REAL,
    created_at TEXT NOT NULL,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_extractions_status ON extractions(extraction_status);
CREATE INDEX IF NOT EXISTS idx_extractions_confidence ON extractions(confidence);
CREATE INDEX IF NOT EXISTS idx_extractions_transcript ON extractions(source_transcript_id);
"#;

/// A flat row representing a draft extraction in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRow {
    pub id: String,
    pub entity_type: String,
    pub title: String,
    pub speech_act: Option<String>,
    pub extraction_status: String,
    pub source_transcript_id: Option<String>,
    pub source_span_start: Option<i64>,
    pub source_span_end: Option<i64>,
    pub confidence: Option<f64>,
    pub created_at: String,
    pub metadata: Option<String>,
}

/// SQLite-backed extraction storage (data/extractions.db).
pub struct ExtractionStore {
    conn: Connection,
}

impl ExtractionStore {
    /// Open or create the extraction store at the given path.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Open an in-memory extraction store (for tests).
    pub fn in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Insert a new draft extraction.
    pub fn insert_draft(&self, row: &ExtractionRow) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO extractions (id, entity_type, title, speech_act, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, created_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                row.id,
                row.entity_type,
                row.title,
                row.speech_act,
                "draft",
                row.source_transcript_id,
                row.source_span_start,
                row.source_span_end,
                row.confidence,
                row.created_at,
                row.metadata,
            ],
        )?;
        Ok(())
    }

    /// Promote a draft extraction. Returns the row so the caller can move it to entities.db.
    pub fn promote(&self, id: &str) -> Result<ExtractionRow, StoreError> {
        let row = self
            .get(id)?
            .ok_or_else(|| StoreError::EntityNotFound(id.to_string()))?;

        if row.extraction_status != "draft" {
            return Err(StoreError::EntityNotFound(format!(
                "extraction {} is not a draft (status: {})",
                id, row.extraction_status
            )));
        }

        self.conn.execute(
            "UPDATE extractions SET extraction_status='promoted' WHERE id=?1",
            params![id],
        )?;

        Ok(ExtractionRow {
            extraction_status: "promoted".to_string(),
            ..row
        })
    }

    /// Discard a draft extraction (removes it from the store).
    pub fn discard(&self, id: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM extractions WHERE id=?1 AND extraction_status='draft'",
            params![id],
        )?;
        Ok(())
    }

    /// Get an extraction by ID.
    pub fn get(&self, id: &str) -> Result<Option<ExtractionRow>, StoreError> {
        let row = self
            .conn
            .query_row(
                "SELECT id, entity_type, title, speech_act, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, created_at, metadata FROM extractions WHERE id=?1",
                params![id],
                |row| row_to_extraction_row(row),
            )
            .optional()?;
        Ok(row)
    }

    /// List all pending (draft) extractions.
    pub fn list_pending(&self) -> Result<Vec<ExtractionRow>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entity_type, title, speech_act, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, created_at, metadata FROM extractions WHERE extraction_status='draft' ORDER BY confidence DESC",
        )?;
        let rows = stmt
            .query_map([], |row| row_to_extraction_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// List draft extractions with confidence above a threshold.
    pub fn list_by_confidence(&self, min: f64) -> Result<Vec<ExtractionRow>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entity_type, title, speech_act, extraction_status, source_transcript_id, source_span_start, source_span_end, confidence, created_at, metadata FROM extractions WHERE extraction_status='draft' AND confidence >= ?1 ORDER BY confidence DESC",
        )?;
        let rows = stmt
            .query_map(params![min], |row| row_to_extraction_row(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

fn row_to_extraction_row(row: &rusqlite::Row) -> rusqlite::Result<ExtractionRow> {
    Ok(ExtractionRow {
        id: row.get(0)?,
        entity_type: row.get(1)?,
        title: row.get(2)?,
        speech_act: row.get(3)?,
        extraction_status: row.get(4)?,
        source_transcript_id: row.get(5)?,
        source_span_start: row.get(6)?,
        source_span_end: row.get(7)?,
        confidence: row.get(8)?,
        created_at: row.get(9)?,
        metadata: row.get(10)?,
    })
}
