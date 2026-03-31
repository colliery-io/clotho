use clotho_store::data::entities::{EntityRow, EntityStore};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// A group of entities in the navigator tree.
#[derive(Debug)]
pub struct EntityGroup {
    pub entity_type: String,
    pub entities: Vec<EntityRow>,
    pub expanded: bool,
}

/// Navigator state — holds the entity list grouped by type.
pub struct Navigator {
    pub groups: Vec<EntityGroup>,
    /// Flat index: (group_idx, entity_idx within group or None for the group header)
    /// Used for cursor navigation.
    pub cursor: usize,
    /// Total visible lines (group headers + expanded entity entries).
    pub visible_count: usize,
    /// Scroll offset for the visible window.
    pub scroll_offset: usize,
    /// Pending expansion state from saved state (applied on next refresh).
    pending_expanded: HashMap<String, bool>,
}

impl Navigator {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            cursor: 0,
            visible_count: 0,
            scroll_offset: 0,
            pending_expanded: HashMap::new(),
        }
    }

    /// Pre-set expansion state for a group (used when restoring from saved state).
    pub fn set_expanded(&mut self, entity_type: &str, expanded: bool) {
        // If the group exists already, set it. Otherwise store for later.
        for group in &mut self.groups {
            if group.entity_type == entity_type {
                group.expanded = expanded;
                return;
            }
        }
        // Store as a pending expansion — will be applied on next refresh
        self.pending_expanded.insert(entity_type.to_string(), expanded);
    }

    /// Reload entities from the store.
    pub fn refresh(&mut self, db_path: &Path) {
        let store = match EntityStore::open(db_path) {
            Ok(s) => s,
            Err(_) => return,
        };

        let all = match store.list_all() {
            Ok(rows) => rows,
            Err(_) => return,
        };

        // Filter to active entities only:
        // - Exclude status=inactive
        // - Exclude task_state=done
        // - Exclude extraction_status=discarded
        let active: Vec<EntityRow> = all
            .into_iter()
            .filter(|row| {
                if let Some(ref status) = row.status {
                    if status.to_lowercase() == "inactive" {
                        return false;
                    }
                }
                if let Some(ref state) = row.task_state {
                    if state.to_lowercase() == "done" {
                        return false;
                    }
                }
                if let Some(ref es) = row.extraction_status {
                    if es.to_lowercase() == "discarded" {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Group by entity_type, preserving a stable order
        let mut grouped: BTreeMap<String, Vec<EntityRow>> = BTreeMap::new();
        for row in active {
            grouped
                .entry(row.entity_type.clone())
                .or_default()
                .push(row);
        }

        // Preserve expansion state from previous groups
        let old_expanded: BTreeMap<String, bool> = self
            .groups
            .iter()
            .map(|g| (g.entity_type.clone(), g.expanded))
            .collect();

        self.groups = grouped
            .into_iter()
            .map(|(entity_type, entities)| {
                let expanded = old_expanded
                    .get(&entity_type)
                    .or_else(|| self.pending_expanded.get(&entity_type))
                    .copied()
                    .unwrap_or(true);
                EntityGroup {
                    entity_type,
                    entities,
                    expanded,
                }
            })
            .collect();
        self.pending_expanded.clear();

        self.recompute_visible_count();

        // Clamp cursor
        if self.visible_count > 0 && self.cursor >= self.visible_count {
            self.cursor = self.visible_count - 1;
        }
    }

    fn recompute_visible_count(&mut self) {
        let mut count = 0;
        for group in &self.groups {
            count += 1; // group header
            if group.expanded {
                count += group.entities.len();
            }
        }
        self.visible_count = count;
    }

    /// Move cursor up.
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor down.
    pub fn cursor_down(&mut self) {
        if self.visible_count > 0 && self.cursor < self.visible_count - 1 {
            self.cursor += 1;
        }
    }

    /// Toggle expand/collapse on the current line if it's a group header.
    pub fn toggle_expand(&mut self) {
        if let Some((group_idx, None)) = self.resolve_cursor() {
            self.groups[group_idx].expanded = !self.groups[group_idx].expanded;
            self.recompute_visible_count();
            if self.cursor >= self.visible_count {
                self.cursor = self.visible_count.saturating_sub(1);
            }
        }
    }

    /// Resolve cursor position to (group_index, Some(entity_index)) or (group_index, None) for header.
    pub fn resolve_cursor(&self) -> Option<(usize, Option<usize>)> {
        let mut pos = 0;
        for (gi, group) in self.groups.iter().enumerate() {
            if pos == self.cursor {
                return Some((gi, None));
            }
            pos += 1;
            if group.expanded {
                for ei in 0..group.entities.len() {
                    if pos == self.cursor {
                        return Some((gi, Some(ei)));
                    }
                    pos += 1;
                }
            }
        }
        None
    }

    /// Get the entity at the current cursor, if it's an entity line.
    pub fn selected_entity(&self) -> Option<&EntityRow> {
        if let Some((gi, Some(ei))) = self.resolve_cursor() {
            Some(&self.groups[gi].entities[ei])
        } else {
            None
        }
    }

    /// Build a list of (line_text, is_header, is_cursor) for rendering.
    pub fn visible_lines(&self, height: usize) -> Vec<(String, bool, bool)> {
        // Adjust scroll to keep cursor visible
        let scroll = {
            let mut s = self.scroll_offset;
            if self.cursor < s {
                s = self.cursor;
            }
            if self.cursor >= s + height {
                s = self.cursor - height + 1;
            }
            s
        };

        let mut lines = Vec::new();
        let mut pos = 0;

        for group in &self.groups {
            if pos >= scroll + height {
                break;
            }

            // Group header
            if pos >= scroll {
                let arrow = if group.expanded { "▾" } else { "▸" };
                let count = group.entities.len();
                let text = format!("{} {} ({})", arrow, group.entity_type, count);
                lines.push((text, true, pos == self.cursor));
            }
            pos += 1;

            if group.expanded {
                for entity in &group.entities {
                    if pos >= scroll + height {
                        break;
                    }
                    if pos >= scroll {
                        let text = format!("  {}", entity.title);
                        lines.push((text, false, pos == self.cursor));
                    }
                    pos += 1;
                }
            }
        }

        // Store updated scroll offset
        // (We can't mutate self here since this is &self, but the caller
        //  should update scroll_offset separately if needed)

        lines
    }

    /// Update scroll offset to keep cursor visible for a given viewport height.
    pub fn adjust_scroll(&mut self, height: usize) {
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        }
        if self.cursor >= self.scroll_offset + height {
            self.scroll_offset = self.cursor - height + 1;
        }
    }
}
