use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::StoreError;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS ontology (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id TEXT NOT NULL,
    category TEXT NOT NULL,
    value TEXT NOT NULL,
    added_at TEXT NOT NULL,
    added_by TEXT,
    UNIQUE(entity_id, category, value)
);

CREATE INDEX IF NOT EXISTS idx_ontology_entity ON ontology(entity_id);
CREATE INDEX IF NOT EXISTS idx_ontology_category ON ontology(category);
CREATE INDEX IF NOT EXISTS idx_ontology_value ON ontology(value);
"#;

/// Valid ontology categories.
pub const CATEGORY_KEYWORD: &str = "keyword";
pub const CATEGORY_SIGNAL_TECHNICAL: &str = "signal_technical";
pub const CATEGORY_SIGNAL_SOCIAL: &str = "signal_social";
pub const CATEGORY_PERSON: &str = "person";
pub const CATEGORY_IGNORE: &str = "ignore";

/// A single ontology entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyEntry {
    pub id: i64,
    pub entity_id: String,
    pub category: String,
    pub value: String,
    pub added_at: String,
    pub added_by: Option<String>,
}

/// The full ontology for an entity, grouped by category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ontology {
    pub entity_id: String,
    pub keywords: Vec<String>,
    pub signal_technical: Vec<String>,
    pub signal_social: Vec<String>,
    pub people: Vec<String>,
    pub ignore: Vec<String>,
}

/// Ontology store backed by a table in entities.db.
pub struct OntologyStore {
    conn: Connection,
}

impl OntologyStore {
    /// Open the ontology store, creating the table if needed.
    /// Uses the same database file as EntityStore (entities.db).
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

    /// Get the full ontology for an entity, grouped by category.
    pub fn get(&self, entity_id: &str) -> Result<Ontology, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT category, value FROM ontology WHERE entity_id = ?1 ORDER BY category, value",
        )?;

        let mut keywords = Vec::new();
        let mut signal_technical = Vec::new();
        let mut signal_social = Vec::new();
        let mut people = Vec::new();
        let mut ignore = Vec::new();

        let rows = stmt.query_map(params![entity_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (category, value) = row?;
            match category.as_str() {
                CATEGORY_KEYWORD => keywords.push(value),
                CATEGORY_SIGNAL_TECHNICAL => signal_technical.push(value),
                CATEGORY_SIGNAL_SOCIAL => signal_social.push(value),
                CATEGORY_PERSON => people.push(value),
                CATEGORY_IGNORE => ignore.push(value),
                _ => {} // ignore unknown categories
            }
        }

        Ok(Ontology {
            entity_id: entity_id.to_string(),
            keywords,
            signal_technical,
            signal_social,
            people,
            ignore,
        })
    }

    /// Add entries to an entity's ontology. Duplicates are silently ignored.
    pub fn add(
        &self,
        entity_id: &str,
        category: &str,
        values: &[&str],
        added_by: Option<&str>,
    ) -> Result<usize, StoreError> {
        let now = Utc::now().to_rfc3339();
        let mut count = 0;

        for value in values {
            let result = self.conn.execute(
                "INSERT OR IGNORE INTO ontology (entity_id, category, value, added_at, added_by) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![entity_id, category, value.trim(), now, added_by],
            )?;
            count += result;
        }

        Ok(count)
    }

    /// Remove entries from an entity's ontology.
    pub fn remove(
        &self,
        entity_id: &str,
        category: &str,
        values: &[&str],
    ) -> Result<usize, StoreError> {
        let mut count = 0;
        for value in values {
            let result = self.conn.execute(
                "DELETE FROM ontology WHERE entity_id = ?1 AND category = ?2 AND value = ?3",
                params![entity_id, category, value.trim()],
            )?;
            count += result;
        }
        Ok(count)
    }

    /// Find which entities have a specific value in their ontology.
    /// Returns entity IDs. Useful for "which programs care about X?"
    pub fn search(&self, value: &str) -> Result<Vec<OntologyEntry>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entity_id, category, value, added_at, added_by FROM ontology WHERE value LIKE ?1 ORDER BY entity_id",
        )?;

        let pattern = format!("%{}%", value);
        let rows = stmt
            .query_map(params![pattern], |row| {
                Ok(OntologyEntry {
                    id: row.get(0)?,
                    entity_id: row.get(1)?,
                    category: row.get(2)?,
                    value: row.get(3)?,
                    added_at: row.get(4)?,
                    added_by: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// Get all entries for an entity as a flat list.
    pub fn list(&self, entity_id: &str) -> Result<Vec<OntologyEntry>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, entity_id, category, value, added_at, added_by FROM ontology WHERE entity_id = ?1 ORDER BY category, value",
        )?;

        let rows = stmt
            .query_map(params![entity_id], |row| {
                Ok(OntologyEntry {
                    id: row.get(0)?,
                    entity_id: row.get(1)?,
                    category: row.get(2)?,
                    value: row.get(3)?,
                    added_at: row.get(4)?,
                    added_by: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}
