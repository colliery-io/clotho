use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("workspace not found at {0}")]
    WorkspaceNotFound(String),

    #[error("workspace already exists at {0}")]
    WorkspaceAlreadyExists(String),

    #[error("invalid workspace structure: {0}")]
    InvalidWorkspace(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("entity not found: {0}")]
    EntityNotFound(String),

    #[error("content not found for {0}")]
    ContentNotFound(String),

    #[error("search error: {0}")]
    SearchError(String),

    #[error("federation error: {0}")]
    FederationError(String),
}
