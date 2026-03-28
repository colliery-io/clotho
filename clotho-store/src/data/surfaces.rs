use std::path::Path;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::StoreError;

/// A surface row in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceRow {
    pub id: String,
    pub title: String,
    pub content: String,
    pub surface_type: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// SQLite-backed surface storage (data/entities.db — same DB as entities).
pub struct SurfaceStore {
    conn: Connection,
}

impl SurfaceStore {
    /// Open the surface store at the given path (same entities.db).
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    /// Create a new surface. Returns the created row.
    pub fn create(
        &self,
        title: &str,
        content: &str,
        surface_type: Option<&str>,
    ) -> Result<SurfaceRow, StoreError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO surfaces (id, title, content, surface_type, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6)",
            params![id, title, content, surface_type, now, now],
        )?;

        Ok(SurfaceRow {
            id,
            title: title.to_string(),
            content: content.to_string(),
            surface_type: surface_type.map(|s| s.to_string()),
            status: "active".to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Create or replace a surface by title. If an active surface with
    /// the same title exists, updates its content. Otherwise creates a new one.
    pub fn push(
        &self,
        title: &str,
        content: &str,
        surface_type: Option<&str>,
        replace: bool,
    ) -> Result<SurfaceRow, StoreError> {
        if replace {
            if let Some(existing) = self.find_active_by_title(title)? {
                self.update_content(&existing.id, content)?;
                return self.get(&existing.id)?.ok_or_else(|| {
                    StoreError::Io(std::io::Error::other("surface disappeared after update"))
                });
            }
        }
        self.create(title, content, surface_type)
    }

    /// Get a surface by ID.
    pub fn get(&self, id: &str) -> Result<Option<SurfaceRow>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, surface_type, status, created_at, updated_at
             FROM surfaces WHERE id = ?1",
        )?;
        let row = stmt
            .query_row(params![id], row_to_surface)
            .optional()?;
        Ok(row)
    }

    /// Find an active surface by exact title.
    pub fn find_active_by_title(&self, title: &str) -> Result<Option<SurfaceRow>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, surface_type, status, created_at, updated_at
             FROM surfaces WHERE title = ?1 AND status = 'active' LIMIT 1",
        )?;
        let row = stmt
            .query_row(params![title], row_to_surface)
            .optional()?;
        Ok(row)
    }

    /// Update surface content.
    pub fn update_content(&self, id: &str, content: &str) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE surfaces SET content = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, content, now],
        )?;
        Ok(())
    }

    /// Close a surface (soft delete).
    pub fn close(&self, id: &str) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE surfaces SET status = 'closed', updated_at = ?2 WHERE id = ?1",
            params![id, now],
        )?;
        Ok(())
    }

    /// List surfaces, optionally filtered by status and/or type.
    pub fn list(
        &self,
        status: Option<&str>,
        surface_type: Option<&str>,
    ) -> Result<Vec<SurfaceRow>, StoreError> {
        let mut sql = String::from(
            "SELECT id, title, content, surface_type, status, created_at, updated_at FROM surfaces WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(s) = status {
            param_values.push(Box::new(s.to_string()));
            sql.push_str(&format!(" AND status = ?{}", param_values.len()));
        }
        if let Some(t) = surface_type {
            param_values.push(Box::new(t.to_string()));
            sql.push_str(&format!(" AND surface_type = ?{}", param_values.len()));
        }

        sql.push_str(" ORDER BY updated_at DESC");

        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(params.as_slice(), row_to_surface)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// List only active surfaces.
    pub fn list_active(&self) -> Result<Vec<SurfaceRow>, StoreError> {
        self.list(Some("active"), None)
    }

    /// Search surfaces by keyword in title or content.
    pub fn search(&self, query: &str) -> Result<Vec<SurfaceRow>, StoreError> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, surface_type, status, created_at, updated_at
             FROM surfaces WHERE title LIKE ?1 OR content LIKE ?1
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt
            .query_map(params![pattern], row_to_surface)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

fn row_to_surface(row: &rusqlite::Row) -> rusqlite::Result<SurfaceRow> {
    Ok(SurfaceRow {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        surface_type: row.get(3)?,
        status: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
