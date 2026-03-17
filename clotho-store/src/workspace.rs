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
            version: "0.0.0".to_string(),
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

/// Visible content directories created at project root.
const VISIBLE_DIRS: &[&str] = &[
    "programs",
    "responsibilities",
    "objectives",
    "workstreams",
    "tasks",
    "meetings",
    "reflections",
    "artifacts",
    "notes",
    "people",
    "derived",
];

/// Machine-managed directories created inside .clotho/.
const HIDDEN_DIRS: &[&str] = &[
    "data",
    "graph",
    "index",
    "inbox",
    "config",
];

/// A Clotho workspace.
///
/// Content lives at the project root (visible, browsable).
/// Machine data lives in `.clotho/` (hidden).
pub struct Workspace {
    /// Path to the `.clotho/` directory.
    pub path: PathBuf,
}

impl Workspace {
    /// Initialize a new workspace at the given path.
    ///
    /// Creates visible content directories at the project root and
    /// machine-managed directories inside `.clotho/`.
    pub fn init(base_path: &Path) -> Result<Self, StoreError> {
        let clotho_path = base_path.join(".clotho");

        if clotho_path.exists() {
            return Err(StoreError::WorkspaceAlreadyExists(
                clotho_path.display().to_string(),
            ));
        }

        // Create visible content directories at project root
        for dir in VISIBLE_DIRS {
            fs::create_dir_all(base_path.join(dir))?;
        }

        // Create hidden machine directories in .clotho/
        for dir in HIDDEN_DIRS {
            fs::create_dir_all(clotho_path.join(dir))?;
        }

        // Write default config.toml
        let config = WorkspaceConfig::default();
        let config_toml = toml::to_string_pretty(&config)?;
        fs::write(clotho_path.join("config/config.toml"), config_toml)?;

        // Write default ontology.toml
        let ontology = OntologyConfig::default();
        let ontology_toml = toml::to_string_pretty(&ontology)?;
        fs::write(clotho_path.join("config/ontology.toml"), ontology_toml)?;

        // Create empty JSONL files
        fs::write(clotho_path.join("data/tags.jsonl"), "")?;
        fs::write(clotho_path.join("data/events.jsonl"), "")?;

        Ok(Self { path: clotho_path })
    }

    /// Open an existing workspace.
    ///
    /// Validates that the .clotho/ directory and essential subdirs exist.
    pub fn open(base_path: &Path) -> Result<Self, StoreError> {
        let clotho_path = base_path.join(".clotho");

        if !clotho_path.exists() {
            return Err(StoreError::WorkspaceNotFound(
                clotho_path.display().to_string(),
            ));
        }

        // Validate essential .clotho/ directories exist
        let required = ["data", "graph", "config"];
        for dir in &required {
            let dir_path = clotho_path.join(dir);
            if !dir_path.is_dir() {
                return Err(StoreError::InvalidWorkspace(format!(
                    "missing directory: .clotho/{}",
                    dir
                )));
            }
        }

        // Validate config files exist
        if !clotho_path.join("config/config.toml").is_file() {
            return Err(StoreError::InvalidWorkspace(
                "missing .clotho/config/config.toml".to_string(),
            ));
        }

        Ok(Self { path: clotho_path })
    }

    /// Path to the project root (parent of .clotho/).
    ///
    /// This is where visible content directories live.
    pub fn project_root(&self) -> PathBuf {
        self.path
            .parent()
            .expect("workspace .clotho/ must have a parent")
            .to_path_buf()
    }

    /// Path to the data directory (.clotho/data/).
    pub fn data_path(&self) -> PathBuf {
        self.path.join("data")
    }

    /// Path to the graph directory (.clotho/graph/).
    pub fn graph_path(&self) -> PathBuf {
        self.path.join("graph")
    }

    /// Path to the index directory (.clotho/index/).
    pub fn index_path(&self) -> PathBuf {
        self.path.join("index")
    }

    /// Path to the inbox directory (.clotho/inbox/).
    pub fn inbox_path(&self) -> PathBuf {
        self.path.join("inbox")
    }

    /// Path to the config directory (.clotho/config/).
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
