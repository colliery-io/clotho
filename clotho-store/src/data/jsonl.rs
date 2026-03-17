use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::StoreError;

// ---------------------------------------------------------------------------
// Tag Store
// ---------------------------------------------------------------------------

/// A tag association: an entity has a tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagEntry {
    pub entity_id: String,
    pub tag: String,
}

/// Append-only JSONL storage for tags (data/tags.jsonl).
pub struct TagStore {
    path: PathBuf,
}

impl TagStore {
    pub fn new(data_path: &Path) -> Self {
        Self {
            path: data_path.join("tags.jsonl"),
        }
    }

    /// Add a tag to an entity. Appends a line to the JSONL file.
    pub fn add_tag(&self, entity_id: &str, tag: &str) -> Result<(), StoreError> {
        // Check for duplicate
        let existing = self.get_tags(entity_id)?;
        if existing.iter().any(|t| t == tag) {
            return Ok(());
        }

        let entry = TagEntry {
            entity_id: entity_id.to_string(),
            tag: tag.to_string(),
        };
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(&entry)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Remove a tag from an entity. Rewrites the file excluding the matching entry.
    pub fn remove_tag(&self, entity_id: &str, tag: &str) -> Result<(), StoreError> {
        let entries = self.read_all()?;
        let filtered: Vec<_> = entries
            .into_iter()
            .filter(|e| !(e.entity_id == entity_id && e.tag == tag))
            .collect();
        self.write_all(&filtered)?;
        Ok(())
    }

    /// Get all tags for an entity.
    pub fn get_tags(&self, entity_id: &str) -> Result<Vec<String>, StoreError> {
        let entries = self.read_all()?;
        let tags = entries
            .into_iter()
            .filter(|e| e.entity_id == entity_id)
            .map(|e| e.tag)
            .collect();
        Ok(tags)
    }

    /// Get all entity IDs that have a specific tag.
    pub fn get_entities_by_tag(&self, tag: &str) -> Result<Vec<String>, StoreError> {
        let entries = self.read_all()?;
        let ids = entries
            .into_iter()
            .filter(|e| e.tag == tag)
            .map(|e| e.entity_id)
            .collect();
        Ok(ids)
    }

    fn read_all(&self) -> Result<Vec<TagEntry>, StoreError> {
        read_jsonl(&self.path)
    }

    fn write_all(&self, entries: &[TagEntry]) -> Result<(), StoreError> {
        write_jsonl(&self.path, entries)
    }
}

// ---------------------------------------------------------------------------
// Event Store
// ---------------------------------------------------------------------------

/// Event types for the activity log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Created,
    Updated,
    Deleted,
    Promoted,
    Discarded,
    Transitioned,
    TagAdded,
    TagRemoved,
}

/// An activity log event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub entity_id: String,
    pub details: Option<serde_json::Value>,
}

/// Append-only JSONL storage for events (data/events.jsonl).
pub struct EventStore {
    path: PathBuf,
}

impl EventStore {
    pub fn new(data_path: &Path) -> Self {
        Self {
            path: data_path.join("events.jsonl"),
        }
    }

    /// Log an event. Appends a line to the JSONL file.
    pub fn log(&self, event: &Event) -> Result<(), StoreError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(event)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Read all events.
    pub fn read_all(&self) -> Result<Vec<Event>, StoreError> {
        read_jsonl(&self.path)
    }
}

// ---------------------------------------------------------------------------
// JSONL helpers
// ---------------------------------------------------------------------------

fn read_jsonl<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, StoreError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut items = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let item: T = serde_json::from_str(trimmed)?;
        items.push(item);
    }

    Ok(items)
}

fn write_jsonl<T: Serialize>(path: &Path, items: &[T]) -> Result<(), StoreError> {
    let mut file = File::create(path)?;
    for item in items {
        let line = serde_json::to_string(item)?;
        writeln!(file, "{}", line)?;
    }
    Ok(())
}
