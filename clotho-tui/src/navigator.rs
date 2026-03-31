use clotho_core::graph::GraphStore;
use clotho_store::data::entities::{EntityRow, EntityStore};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// A navigable section in the tree.
#[derive(Debug)]
pub struct NavSection {
    pub title: String,
    pub items: Vec<NavItem>,
    pub expanded: bool,
}

/// An item within a section — either a subsection or a leaf entity.
#[derive(Debug)]
pub enum NavItem {
    /// A subsection (e.g., "Doing" under Tasks, or a Program's children grouped by type)
    SubSection {
        title: String,
        entities: Vec<EntityRow>,
        expanded: bool,
    },
    /// A leaf entity with no nesting.
    Entity(EntityRow),
}

/// Navigator state.
pub struct Navigator {
    pub sections: Vec<NavSection>,
    pub cursor: usize,
    pub visible_count: usize,
    pub scroll_offset: usize,
    /// Show archived/inactive entities.
    pub show_archived: bool,
    /// Search mode.
    pub searching: bool,
    pub search_query: String,
    search_results: Vec<EntityRow>,
    /// Tracks expansion state by key (persisted across refreshes).
    expansion_state: HashMap<String, bool>,
}

impl Navigator {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
            cursor: 0,
            visible_count: 0,
            scroll_offset: 0,
            show_archived: false,
            searching: false,
            search_query: String::new(),
            search_results: Vec::new(),
            expansion_state: HashMap::new(),
        }
    }

    pub fn set_expanded(&mut self, key: &str, expanded: bool) {
        self.expansion_state.insert(key.to_string(), expanded);
    }

    fn is_expanded(&self, key: &str, default: bool) -> bool {
        self.expansion_state.get(key).copied().unwrap_or(default)
    }

    fn save_expansion(&mut self) {
        for section in &self.sections {
            self.expansion_state.insert(section.title.clone(), section.expanded);
            for item in &section.items {
                if let NavItem::SubSection { title, expanded, .. } = item {
                    self.expansion_state.insert(title.clone(), *expanded);
                }
            }
        }
    }

    /// Reload from entity store and graph.
    pub fn refresh(&mut self, db_path: &Path) {
        let store = match EntityStore::open(db_path) {
            Ok(s) => s,
            Err(_) => return,
        };

        let all = match store.list_all() {
            Ok(rows) => rows,
            Err(_) => return,
        };

        // Filter if not showing archived
        let entities: Vec<EntityRow> = if self.show_archived {
            all
        } else {
            all.into_iter()
                .filter(|row| {
                    if let Some(ref status) = row.status {
                        if status.to_lowercase() == "inactive" { return false; }
                    }
                    if let Some(ref es) = row.extraction_status {
                        if es.to_lowercase() == "discarded" { return false; }
                    }
                    true
                })
                .collect()
        };

        // Load belongs_to relations from graph
        let graph_path = db_path.parent().unwrap_or(Path::new(".")).join("../graph/relations.db");
        let parent_map = load_parent_map(&graph_path);

        // Save current expansion state before rebuilding
        self.save_expansion();

        // Build entity lookup
        let entity_map: HashMap<String, &EntityRow> = entities.iter()
            .map(|e| (e.id.clone(), e))
            .collect();

        // Categorize entities
        let mut tasks_doing = Vec::new();
        let mut tasks_blocked = Vec::new();
        let mut tasks_todo = Vec::new();
        let mut tasks_done = Vec::new();
        let mut risks = Vec::new();
        let mut blockers = Vec::new();
        let mut programs: BTreeMap<String, (EntityRow, Vec<EntityRow>)> = BTreeMap::new();
        let mut responsibilities: BTreeMap<String, (EntityRow, Vec<EntityRow>)> = BTreeMap::new();
        let mut people = Vec::new();
        let mut accounted: HashSet<String> = HashSet::new();

        // First pass: identify programs, responsibilities, people
        for entity in &entities {
            match entity.entity_type.as_str() {
                "Program" => {
                    programs.insert(entity.id.clone(), (entity.clone(), Vec::new()));
                    accounted.insert(entity.id.clone());
                }
                "Responsibility" => {
                    responsibilities.insert(entity.id.clone(), (entity.clone(), Vec::new()));
                    accounted.insert(entity.id.clone());
                }
                "Person" => {
                    people.push(entity.clone());
                    accounted.insert(entity.id.clone());
                }
                _ => {}
            }
        }

        // Second pass: categorize tasks, risks, blockers, and program children
        for entity in &entities {
            if accounted.contains(&entity.id) { continue; }

            match entity.entity_type.as_str() {
                "Task" => {
                    accounted.insert(entity.id.clone());
                    match entity.task_state.as_deref().map(|s| s.to_lowercase()).as_deref() {
                        Some("doing") => tasks_doing.push(entity.clone()),
                        Some("blocked") => tasks_blocked.push(entity.clone()),
                        Some("done") => tasks_done.push(entity.clone()),
                        _ => tasks_todo.push(entity.clone()),
                    }
                }
                "Risk" => {
                    risks.push(entity.clone());
                    accounted.insert(entity.id.clone());
                }
                "Blocker" => {
                    blockers.push(entity.clone());
                    accounted.insert(entity.id.clone());
                }
                _ => {
                    // Try to assign to a program or responsibility via belongs_to
                    if let Some(parent_id) = parent_map.get(&entity.id) {
                        if let Some((_, children)) = programs.get_mut(parent_id) {
                            children.push(entity.clone());
                            accounted.insert(entity.id.clone());
                        } else if let Some((_, children)) = responsibilities.get_mut(parent_id) {
                            children.push(entity.clone());
                            accounted.insert(entity.id.clone());
                        }
                    }
                }
            }
        }

        // Build sections
        let mut sections = Vec::new();

        // Tasks section
        let mut task_items = Vec::new();
        if !tasks_doing.is_empty() {
            task_items.push(NavItem::SubSection {
                title: format!("Doing ({})", tasks_doing.len()),
                entities: tasks_doing,
                expanded: self.is_expanded(&format!("Doing ("), true),
            });
        }
        if !tasks_blocked.is_empty() {
            task_items.push(NavItem::SubSection {
                title: format!("Blocked ({})", tasks_blocked.len()),
                entities: tasks_blocked,
                expanded: self.is_expanded(&format!("Blocked ("), true),
            });
        }
        if !tasks_todo.is_empty() {
            task_items.push(NavItem::SubSection {
                title: format!("Todo ({})", tasks_todo.len()),
                entities: tasks_todo,
                expanded: self.is_expanded(&format!("Todo ("), true),
            });
        }
        if !tasks_done.is_empty() {
            task_items.push(NavItem::SubSection {
                title: format!("Done ({})", tasks_done.len()),
                entities: tasks_done,
                expanded: self.is_expanded(&format!("Done ("), false), // collapsed by default
            });
        }
        if !task_items.is_empty() {
            sections.push(NavSection {
                title: "Tasks".to_string(),
                items: task_items,
                expanded: self.is_expanded("Tasks", true),
            });
        }

        // Risks & Blockers section
        let mut rb_items: Vec<NavItem> = Vec::new();
        for r in risks { rb_items.push(NavItem::Entity(r)); }
        for b in blockers { rb_items.push(NavItem::Entity(b)); }
        if !rb_items.is_empty() {
            sections.push(NavSection {
                title: "Risks & Blockers".to_string(),
                items: rb_items,
                expanded: self.is_expanded("Risks & Blockers", true),
            });
        }

        // Programs section
        let mut prog_items = Vec::new();
        for (_id, (program, children)) in &programs {
            if children.is_empty() {
                prog_items.push(NavItem::Entity(program.clone()));
            } else {
                let mut child_entities: Vec<EntityRow> = children.clone();
                child_entities.sort_by(|a, b| a.entity_type.cmp(&b.entity_type).then(a.title.cmp(&b.title)));
                prog_items.push(NavItem::SubSection {
                    title: program.title.clone(),
                    entities: child_entities,
                    expanded: self.is_expanded(&program.title, false),
                });
            }
        }
        if !prog_items.is_empty() {
            sections.push(NavSection {
                title: "Programs".to_string(),
                items: prog_items,
                expanded: self.is_expanded("Programs", true),
            });
        }

        // Responsibilities section
        let mut resp_items = Vec::new();
        for (_id, (resp, children)) in &responsibilities {
            if children.is_empty() {
                resp_items.push(NavItem::Entity(resp.clone()));
            } else {
                let mut child_entities: Vec<EntityRow> = children.clone();
                child_entities.sort_by(|a, b| a.entity_type.cmp(&b.entity_type).then(a.title.cmp(&b.title)));
                resp_items.push(NavItem::SubSection {
                    title: resp.title.clone(),
                    entities: child_entities,
                    expanded: self.is_expanded(&resp.title, false),
                });
            }
        }
        if !resp_items.is_empty() {
            sections.push(NavSection {
                title: "Responsibilities".to_string(),
                items: resp_items,
                expanded: self.is_expanded("Responsibilities", true),
            });
        }

        // People section
        if !people.is_empty() {
            let people_items = people.into_iter().map(NavItem::Entity).collect();
            sections.push(NavSection {
                title: "People".to_string(),
                items: people_items,
                expanded: self.is_expanded("People", true),
            });
        }

        // Unlinked section
        let unlinked: Vec<EntityRow> = entities.iter()
            .filter(|e| !accounted.contains(&e.id))
            .cloned()
            .collect();
        if !unlinked.is_empty() {
            let unlinked_items = unlinked.into_iter().map(NavItem::Entity).collect();
            sections.push(NavSection {
                title: "Unlinked".to_string(),
                items: unlinked_items,
                expanded: self.is_expanded("Unlinked", false),
            });
        }

        self.sections = sections;
        self.recompute_visible_count();

        if self.visible_count > 0 && self.cursor >= self.visible_count {
            self.cursor = self.visible_count - 1;
        }
    }

    fn recompute_visible_count(&mut self) {
        self.visible_count = 0;
        for section in &self.sections {
            self.visible_count += 1; // section header
            if section.expanded {
                for item in &section.items {
                    match item {
                        NavItem::Entity(_) => self.visible_count += 1,
                        NavItem::SubSection { entities, expanded, .. } => {
                            self.visible_count += 1; // subsection header
                            if *expanded {
                                self.visible_count += entities.len();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_down(&mut self) {
        let max = if self.searching { self.search_results.len() } else { self.visible_count };
        if max > 0 && self.cursor < max - 1 {
            self.cursor += 1;
        }
    }

    /// Toggle expand/collapse at cursor.
    pub fn toggle_expand(&mut self) {
        if let Some(pos) = self.resolve_cursor_position() {
            match pos {
                CursorPosition::SectionHeader(si) => {
                    self.sections[si].expanded = !self.sections[si].expanded;
                    self.recompute_visible_count();
                    if self.cursor >= self.visible_count {
                        self.cursor = self.visible_count.saturating_sub(1);
                    }
                }
                CursorPosition::SubSectionHeader(si, ii) => {
                    if let NavItem::SubSection { expanded, .. } = &mut self.sections[si].items[ii] {
                        *expanded = !*expanded;
                        self.recompute_visible_count();
                        if self.cursor >= self.visible_count {
                            self.cursor = self.visible_count.saturating_sub(1);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Get the entity at the cursor.
    pub fn selected_entity(&self) -> Option<&EntityRow> {
        match self.resolve_cursor_position()? {
            CursorPosition::Entity(_, entity) => Some(entity),
            CursorPosition::SubSectionEntity(_, _, entity) => Some(entity),
            CursorPosition::TopLevelEntity(_, entity) => Some(entity),
            _ => None,
        }
    }

    /// Resolve what the cursor is pointing at.
    pub fn resolve_cursor(&self) -> Option<(usize, Option<usize>)> {
        // Legacy compat — returns (section, None) for headers
        match self.resolve_cursor_position()? {
            CursorPosition::SectionHeader(si) => Some((si, None)),
            CursorPosition::SubSectionHeader(si, _) => Some((si, None)),
            _ => Some((0, Some(0))), // entity — non-None second element
        }
    }

    fn resolve_cursor_position(&self) -> Option<CursorPosition> {
        let mut pos = 0;
        for (si, section) in self.sections.iter().enumerate() {
            if pos == self.cursor {
                return Some(CursorPosition::SectionHeader(si));
            }
            pos += 1;

            if section.expanded {
                for (ii, item) in section.items.iter().enumerate() {
                    match item {
                        NavItem::Entity(entity) => {
                            if pos == self.cursor {
                                return Some(CursorPosition::TopLevelEntity(si, entity));
                            }
                            pos += 1;
                        }
                        NavItem::SubSection { entities, expanded, .. } => {
                            if pos == self.cursor {
                                return Some(CursorPosition::SubSectionHeader(si, ii));
                            }
                            pos += 1;
                            if *expanded {
                                for entity in entities {
                                    if pos == self.cursor {
                                        return Some(CursorPosition::SubSectionEntity(si, ii, entity));
                                    }
                                    pos += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Build visible lines for rendering.
    pub fn visible_lines(&self, height: usize) -> Vec<(String, bool, bool)> {
        let scroll = self.scroll_offset;
        let mut lines = Vec::new();
        let mut pos = 0;

        for section in &self.sections {
            if pos >= scroll + height { break; }

            // Section header
            if pos >= scroll {
                let arrow = if section.expanded { "▾" } else { "▸" };
                let text = format!("{} {}", arrow, section.title);
                lines.push((text, true, pos == self.cursor));
            }
            pos += 1;

            if section.expanded {
                for item in &section.items {
                    if pos >= scroll + height { break; }

                    match item {
                        NavItem::Entity(entity) => {
                            if pos >= scroll {
                                let text = format!("  {}", entity.title);
                                lines.push((text, false, pos == self.cursor));
                            }
                            pos += 1;
                        }
                        NavItem::SubSection { title, entities, expanded } => {
                            if pos >= scroll {
                                let arrow = if *expanded { "▾" } else { "▸" };
                                let text = format!("  {} {}", arrow, title);
                                lines.push((text, true, pos == self.cursor));
                            }
                            pos += 1;

                            if *expanded {
                                for entity in entities {
                                    if pos >= scroll + height { break; }
                                    if pos >= scroll {
                                        let text = format!("    {}", entity.title);
                                        lines.push((text, false, pos == self.cursor));
                                    }
                                    pos += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        lines
    }

    pub fn adjust_scroll(&mut self, height: usize) {
        let max = if self.searching { self.search_results.len() } else { self.visible_count };
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        }
        if max > 0 && self.cursor >= self.scroll_offset + height {
            self.scroll_offset = self.cursor - height + 1;
        }
    }

    // --- Search ---

    pub fn start_search(&mut self) {
        self.searching = true;
        self.search_query.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
        self.update_search_results();
    }

    pub fn stop_search(&mut self) {
        self.searching = false;
        self.search_query.clear();
        self.search_results.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
    }

    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
        self.cursor = 0;
        self.scroll_offset = 0;
        self.update_search_results();
    }

    pub fn search_pop(&mut self) {
        self.search_query.pop();
        self.cursor = 0;
        self.scroll_offset = 0;
        self.update_search_results();
    }

    fn update_search_results(&mut self) {
        self.search_results.clear();
        let query = self.search_query.to_lowercase();

        for section in &self.sections {
            for item in &section.items {
                match item {
                    NavItem::Entity(entity) => {
                        if query.is_empty() || entity.title.to_lowercase().contains(&query) {
                            self.search_results.push(entity.clone());
                        }
                    }
                    NavItem::SubSection { entities, .. } => {
                        for entity in entities {
                            if query.is_empty() || entity.title.to_lowercase().contains(&query) {
                                self.search_results.push(entity.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn selected_search_entity(&self) -> Option<&EntityRow> {
        self.search_results.get(self.cursor)
    }

    pub fn search_lines(&self, height: usize) -> Vec<(String, bool, bool)> {
        let scroll = self.scroll_offset;
        let mut lines = Vec::new();

        for (pos, entity) in self.search_results.iter().enumerate() {
            if pos >= scroll + height { break; }
            if pos >= scroll {
                let text = format!("  {} | {}", entity.entity_type, entity.title);
                lines.push((text, false, pos == self.cursor));
            }
        }

        lines
    }
}

enum CursorPosition<'a> {
    SectionHeader(usize),
    SubSectionHeader(usize, usize),
    TopLevelEntity(usize, &'a EntityRow),
    SubSectionEntity(usize, usize, &'a EntityRow),
    #[allow(dead_code)]
    Entity(usize, &'a EntityRow),
}

/// Load all belongs_to relations: child_id -> parent_id
fn load_parent_map(graph_path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let graph = match GraphStore::open(graph_path) {
        Ok(g) => g,
        Err(_) => return map,
    };

    let query = "MATCH (a)-[:BELONGS_TO]->(b) RETURN a.id AS child, b.id AS parent";
    if let Ok(result) = graph.graph().connection().cypher(query) {
        for row in result.iter() {
            let child: String = row.get("child").unwrap_or_default();
            let parent: String = row.get("parent").unwrap_or_default();
            if !child.is_empty() && !parent.is_empty() {
                map.insert(child, parent);
            }
        }
    }

    map
}
