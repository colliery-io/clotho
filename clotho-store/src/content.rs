use std::fs;
use std::path::{Path, PathBuf};

use clotho_core::domain::types::{EntityId, EntityType};

use crate::error::StoreError;

/// Manages markdown content files at the project root.
///
/// Content directories are visible and browsable (not hidden in .clotho/).
/// Each entity type maps to a directory at the project root:
/// programs/, responsibilities/, objectives/, workstreams/, tasks/,
/// meetings/, reflections/, artifacts/, notes/, people/, derived/
pub struct ContentStore {
    /// Project root path (parent of .clotho/).
    project_root: PathBuf,
}

impl ContentStore {
    /// Create a new ContentStore rooted at the project directory.
    ///
    /// The project root is the parent of .clotho/ — where visible
    /// content directories live.
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    /// Resolve the filesystem path for an entity's content file (no I/O).
    pub fn content_path(&self, entity_type: EntityType, id: &EntityId) -> PathBuf {
        let subdir = entity_type_to_subdir(entity_type);
        self.project_root.join(subdir).join(format!("{}.md", id))
    }

    /// Write markdown content for an entity.
    ///
    /// Creates the file (and parent directories if needed). Returns the path written to.
    pub fn write_content(
        &self,
        entity_type: EntityType,
        id: &EntityId,
        content: &str,
    ) -> Result<PathBuf, StoreError> {
        let path = self.content_path(entity_type, id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Read markdown content for an entity.
    ///
    /// Returns `None` if the file doesn't exist.
    pub fn read_content(
        &self,
        entity_type: EntityType,
        id: &EntityId,
    ) -> Result<Option<String>, StoreError> {
        let path = self.content_path(entity_type, id);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// Delete the content file for an entity.
    pub fn delete_content(&self, entity_type: EntityType, id: &EntityId) -> Result<(), StoreError> {
        let path = self.content_path(entity_type, id);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// List all content files in a subdirectory for a given entity type.
    pub fn list_content(&self, entity_type: EntityType) -> Result<Vec<PathBuf>, StoreError> {
        let subdir = entity_type_to_subdir(entity_type);
        let dir = self.project_root.join(subdir);

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }
        Ok(files)
    }
}

/// Map an EntityType to its visible content directory name at project root.
fn entity_type_to_subdir(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Program => "programs",
        EntityType::Responsibility => "responsibilities",
        EntityType::Objective => "objectives",
        EntityType::Workstream => "workstreams",
        EntityType::Task => "tasks",
        EntityType::Meeting | EntityType::Transcript => "meetings",
        EntityType::Reflection => "reflections",
        EntityType::Artifact => "artifacts",
        EntityType::Reference => "references",
        EntityType::Note => "notes",
        EntityType::Person => "people",
        EntityType::Decision
        | EntityType::Risk
        | EntityType::Blocker
        | EntityType::Question
        | EntityType::Insight => "derived",
    }
}
