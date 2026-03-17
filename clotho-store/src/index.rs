use std::path::Path;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::content::ContentStore;
use crate::data::entities::EntityStore;
use crate::error::StoreError;

const SCHEMA: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
    entity_id,
    entity_type,
    title,
    content,
    tokenize='porter unicode61'
);
"#;

/// A search result from the FTS5 index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity_id: String,
    pub entity_type: String,
    pub title: String,
    pub snippet: String,
    pub rank: f64,
}

/// FTS5-backed keyword search index (index/search.db).
///
/// Fully derived — can be rebuilt from entities.db and content files at any time.
pub struct SearchIndex {
    conn: Connection,
}

impl SearchIndex {
    /// Open or create the search index at the given path.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Open an in-memory search index (for tests).
    pub fn in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Index an entity's content.
    pub fn index_entity(
        &self,
        entity_id: &str,
        entity_type: &str,
        title: &str,
        content: &str,
    ) -> Result<(), StoreError> {
        // Remove existing entry first (upsert semantics)
        self.remove_entity(entity_id)?;

        self.conn.execute(
            "INSERT INTO search_index (entity_id, entity_type, title, content) VALUES (?1, ?2, ?3, ?4)",
            params![entity_id, entity_type, title, content],
        )?;
        Ok(())
    }

    /// Remove an entity from the search index.
    pub fn remove_entity(&self, entity_id: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM search_index WHERE entity_id = ?1",
            params![entity_id],
        )?;
        Ok(())
    }

    /// Search by keyword using FTS5 MATCH with BM25 ranking.
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, StoreError> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = self.conn.prepare(
            "SELECT entity_id, entity_type, title, snippet(search_index, 3, '<b>', '</b>', '...', 32) AS snip, rank
             FROM search_index
             WHERE search_index MATCH ?1
             ORDER BY rank",
        )?;

        let results = stmt
            .query_map(params![query], |row| {
                Ok(SearchResult {
                    entity_id: row.get(0)?,
                    entity_type: row.get(1)?,
                    title: row.get(2)?,
                    snippet: row.get(3)?,
                    rank: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| StoreError::SearchError(e.to_string()))?;

        Ok(results)
    }

    /// Drop and rebuild the entire index from entities.db and content files.
    pub fn rebuild(
        &self,
        entity_store: &EntityStore,
        _content_store: &ContentStore,
    ) -> Result<usize, StoreError> {
        // Clear existing index
        self.conn
            .execute("DELETE FROM search_index", [])?;

        let entities = entity_store.list_all()?;
        let mut count = 0;

        for entity in &entities {
            // Try to read content from the content store
            let content = if let Some(content_path) = &entity.content_path {
                // Read from the actual file if content_path is set
                let path = std::path::Path::new(content_path);
                if path.exists() {
                    std::fs::read_to_string(path).unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            self.index_entity(
                &entity.id,
                &entity.entity_type,
                &entity.title,
                &content,
            )?;
            count += 1;
        }

        Ok(count)
    }

    /// Access the underlying connection (for federation ATTACH).
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}
