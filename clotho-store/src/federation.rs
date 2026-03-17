use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde_json::Value;

use crate::error::StoreError;

/// Thin federation layer using SQLite ATTACH DATABASE (CLO-A-0002).
///
/// Enables cross-database queries spanning entities.db, relations.db,
/// and search.db from a single connection.
pub struct Federation {
    entities_db: PathBuf,
    relations_db: PathBuf,
    search_db: PathBuf,
}

/// A row from a federation query, represented as a map of column names to values.
pub type FederationRow = std::collections::HashMap<String, Value>;

impl Federation {
    /// Create a federation handle for a workspace.
    pub fn open(workspace_path: &Path) -> Result<Self, StoreError> {
        let entities_db = workspace_path.join("data/entities.db");
        let relations_db = workspace_path.join("graph/relations.db");
        let search_db = workspace_path.join("index/search.db");

        // Validate at least entities.db exists (others may not exist yet)
        if !entities_db.exists() {
            return Err(StoreError::FederationError(format!(
                "entities.db not found at {}",
                entities_db.display()
            )));
        }

        Ok(Self {
            entities_db,
            relations_db,
            search_db,
        })
    }

    /// Create a connection with all databases attached.
    ///
    /// - Main database: entities.db (as "ent")
    /// - Attached: relations.db (as "graph"), search.db (as "idx")
    pub fn connect(&self) -> Result<Connection, StoreError> {
        let conn = Connection::open(&self.entities_db)?;

        // Attach relations.db if it exists
        if self.relations_db.exists() {
            conn.execute(
                &format!(
                    "ATTACH DATABASE '{}' AS graph",
                    self.relations_db.display()
                ),
                [],
            )
            .map_err(|e| StoreError::FederationError(format!("attach relations.db: {}", e)))?;
        }

        // Attach search.db if it exists
        if self.search_db.exists() {
            conn.execute(
                &format!(
                    "ATTACH DATABASE '{}' AS idx",
                    self.search_db.display()
                ),
                [],
            )
            .map_err(|e| StoreError::FederationError(format!("attach search.db: {}", e)))?;
        }

        Ok(conn)
    }

    /// Execute a cross-database SQL query and return results as rows of JSON values.
    pub fn query(&self, sql: &str) -> Result<Vec<FederationRow>, StoreError> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| StoreError::FederationError(e.to_string()))?;

        let column_names: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let rows = stmt
            .query_map([], |row| {
                let mut map = FederationRow::new();
                for (i, name) in column_names.iter().enumerate() {
                    let val: rusqlite::types::Value = row.get(i)?;
                    let json_val = sqlite_to_json(val);
                    map.insert(name.clone(), json_val);
                }
                Ok(map)
            })
            .map_err(|e| StoreError::FederationError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| StoreError::FederationError(e.to_string()))?;

        Ok(rows)
    }
}

/// Convert a rusqlite Value to a serde_json Value.
fn sqlite_to_json(val: rusqlite::types::Value) -> Value {
    match val {
        rusqlite::types::Value::Null => Value::Null,
        rusqlite::types::Value::Integer(i) => Value::Number(i.into()),
        rusqlite::types::Value::Real(f) => {
            serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        rusqlite::types::Value::Text(s) => Value::String(s),
        rusqlite::types::Value::Blob(b) => {
            Value::String(format!("<blob:{} bytes>", b.len()))
        }
    }
}
