pub mod edges;
pub mod nodes;
pub mod queries;

use std::path::Path;

use graphqlite::Graph;

use crate::error::GraphError;

/// The core graph store wrapping graphqlite.
///
/// Provides typed CRUD operations for entity nodes and relation edges.
/// File-backed for production use, in-memory for tests.
pub struct GraphStore {
    inner: Graph,
}

impl GraphStore {
    /// Open a file-backed graph database.
    pub fn open(path: &Path) -> Result<Self, GraphError> {
        let graph = Graph::open(path).map_err(|e| GraphError::OpenFailed(e.to_string()))?;
        Ok(Self { inner: graph })
    }

    /// Create an in-memory graph database (for tests).
    pub fn in_memory() -> Result<Self, GraphError> {
        let graph = Graph::open_in_memory().map_err(|e| GraphError::OpenFailed(e.to_string()))?;
        Ok(Self { inner: graph })
    }

    /// Access the underlying graphqlite Graph for advanced operations.
    pub fn graph(&self) -> &Graph {
        &self.inner
    }
}
