use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::StoreError;

/// Default configuration for a Clotho workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub version: String,
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub auto_commit: bool,
    pub debounce_seconds: u64,
    pub shallow_history_limit: u32,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            sync: SyncConfig {
                auto_commit: true,
                debounce_seconds: 30,
                shallow_history_limit: 20,
            },
        }
    }
}

/// Default ontology configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyConfig {
    pub known_entities: Vec<String>,
    pub extraction: ExtractionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    pub default_confidence_threshold: f32,
}

impl Default for OntologyConfig {
    fn default() -> Self {
        Self {
            known_entities: Vec::new(),
            extraction: ExtractionConfig {
                default_confidence_threshold: 0.5,
            },
        }
    }
}

/// A Clotho workspace rooted at a `.workspace/` directory.
pub struct Workspace {
    /// Path to the `.workspace/` root.
    pub path: PathBuf,
}

impl Workspace {
    /// Initialize a new workspace at the given path.
    ///
    /// Creates the full directory tree and default config files.
    /// The path should be where `.workspace/` will be created
    /// (e.g., passing `/home/user/work` creates `/home/user/work/.workspace/`).
    pub fn init(base_path: &Path) -> Result<Self, StoreError> {
        let workspace_path = base_path.join(".workspace");

        if workspace_path.exists() {
            return Err(StoreError::WorkspaceAlreadyExists(
                workspace_path.display().to_string(),
            ));
        }

        // Create directory tree
        let dirs = [
            "content/meetings",
            "content/reflections",
            "content/artifacts",
            "content/notes",
            "content/people",
            "data",
            "graph",
            "index",
            "config",
        ];

        for dir in &dirs {
            fs::create_dir_all(workspace_path.join(dir))?;
        }

        // Write default config.toml
        let config = WorkspaceConfig::default();
        let config_toml = toml::to_string_pretty(&config)?;
        fs::write(workspace_path.join("config/config.toml"), config_toml)?;

        // Write default ontology.toml
        let ontology = OntologyConfig::default();
        let ontology_toml = toml::to_string_pretty(&ontology)?;
        fs::write(workspace_path.join("config/ontology.toml"), ontology_toml)?;

        // Create empty JSONL files
        fs::write(workspace_path.join("data/tags.jsonl"), "")?;
        fs::write(workspace_path.join("data/events.jsonl"), "")?;

        Ok(Self {
            path: workspace_path,
        })
    }

    /// Open an existing workspace.
    ///
    /// Validates that the directory structure is intact.
    pub fn open(base_path: &Path) -> Result<Self, StoreError> {
        let workspace_path = base_path.join(".workspace");

        if !workspace_path.exists() {
            return Err(StoreError::WorkspaceNotFound(
                workspace_path.display().to_string(),
            ));
        }

        // Validate essential directories exist
        let required = ["content", "data", "graph", "config"];
        for dir in &required {
            let dir_path = workspace_path.join(dir);
            if !dir_path.is_dir() {
                return Err(StoreError::InvalidWorkspace(format!(
                    "missing directory: {}",
                    dir
                )));
            }
        }

        // Validate config files exist
        if !workspace_path.join("config/config.toml").is_file() {
            return Err(StoreError::InvalidWorkspace(
                "missing config/config.toml".to_string(),
            ));
        }

        Ok(Self {
            path: workspace_path,
        })
    }

    /// Path to the content directory.
    pub fn content_path(&self) -> PathBuf {
        self.path.join("content")
    }

    /// Path to the data directory.
    pub fn data_path(&self) -> PathBuf {
        self.path.join("data")
    }

    /// Path to the graph directory.
    pub fn graph_path(&self) -> PathBuf {
        self.path.join("graph")
    }

    /// Path to the index directory.
    pub fn index_path(&self) -> PathBuf {
        self.path.join("index")
    }

    /// Path to the config directory.
    pub fn config_path(&self) -> PathBuf {
        self.path.join("config")
    }

    /// Read the workspace configuration.
    pub fn read_config(&self) -> Result<WorkspaceConfig, StoreError> {
        let config_str = fs::read_to_string(self.config_path().join("config.toml"))?;
        let config: WorkspaceConfig = toml::from_str(&config_str)?;
        Ok(config)
    }

    /// Read the ontology configuration.
    pub fn read_ontology(&self) -> Result<OntologyConfig, StoreError> {
        let ontology_str = fs::read_to_string(self.config_path().join("ontology.toml"))?;
        let ontology: OntologyConfig = toml::from_str(&ontology_str)?;
        Ok(ontology)
    }
}
