use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::StoreError;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS processing_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL,
    process_name TEXT NOT NULL,
    ontology_ids TEXT,
    processed_at TEXT NOT NULL,
    processed_by TEXT,
    output_entity_ids TEXT,
    notes TEXT,
    UNIQUE(entity_id, process_name, ontology_ids)
);

CREATE INDEX IF NOT EXISTS idx_processing_entity ON processing_log(entity_id);
CREATE INDEX IF NOT EXISTS idx_processing_name ON processing_log(process_name);
"#;

/// A record of a process that was run against an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingRecord {
    pub id: i64,
    pub entity_id: String,
    /// Name of the process: "extraction", "summarization", etc.
    pub process_name: String,
    /// Comma-separated ontology IDs used (if applicable).
    pub ontology_ids: Option<String>,
    pub processed_at: String,
    /// Who ran it: "debrief-processor", "transcript-ingestor", "user", etc.
    pub processed_by: Option<String>,
    /// Comma-separated entity IDs that were created as output.
    pub output_entity_ids: Option<String>,
    /// Freeform notes about the processing.
    pub notes: Option<String>,
}

/// Processing log backed by a table in entities.db.
pub struct ProcessingLog {
    conn: Connection,
}

impl ProcessingLog {
    /// Open the processing log, creating the table if needed.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Open an in-memory store (for tests).
    pub fn in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Record that a process was run against an entity.
    /// Returns false if this exact process+ontology combination was already recorded (idempotent).
    pub fn record(
        &self,
        entity_id: &str,
        process_name: &str,
        ontology_ids: Option<&str>,
        processed_by: Option<&str>,
        output_entity_ids: Option<&str>,
        notes: Option<&str>,
    ) -> Result<bool, StoreError> {
        let now = Utc::now().to_rfc3339();
        let result = self.conn.execute(
            "INSERT OR IGNORE INTO processing_log (entity_id, process_name, ontology_ids, processed_at, processed_by, output_entity_ids, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![entity_id, process_name, ontology_ids, now, processed_by, output_entity_ids, notes],
        )?;
        Ok(result > 0) // true if inserted, false if already existed
    }

    /// Check if an entity has been processed by a specific process.
    pub fn was_processed(&self, entity_id: &str, process_name: &str) -> Result<bool, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT count(*) FROM processing_log WHERE entity_id = ?1 AND process_name = ?2",
            params![entity_id, process_name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Check if an entity has been processed with a specific ontology.
    pub fn was_processed_with_ontology(
        &self,
        entity_id: &str,
        process_name: &str,
        ontology_id: &str,
    ) -> Result<bool, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT count(*) FROM processing_log WHERE entity_id = ?1 AND process_name = ?2 AND ontology_ids LIKE ?3",
            params![entity_id, process_name, format!("%{}%", ontology_id)],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get all processing records for an entity.
    pub fn get_history(&self, entity_id: &str) -> Result<Vec<ProcessingRecord>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entity_id, process_name, ontology_ids, processed_at, processed_by, output_entity_ids, notes
             FROM processing_log WHERE entity_id = ?1 ORDER BY processed_at DESC",
        )?;
        let rows = stmt
            .query_map(params![entity_id], |row| {
                Ok(ProcessingRecord {
                    id: row.get(0)?,
                    entity_id: row.get(1)?,
                    process_name: row.get(2)?,
                    ontology_ids: row.get(3)?,
                    processed_at: row.get(4)?,
                    processed_by: row.get(5)?,
                    output_entity_ids: row.get(6)?,
                    notes: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Get all unprocessed entities of a given type (entities with no processing record for a given process).
    pub fn get_unprocessed(
        &self,
        process_name: &str,
        entity_ids: &[&str],
    ) -> Result<Vec<String>, StoreError> {
        let mut unprocessed = Vec::new();
        for id in entity_ids {
            if !self.was_processed(id, process_name)? {
                unprocessed.push(id.to_string());
            }
        }
        Ok(unprocessed)
    }
}
