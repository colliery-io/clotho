use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui_textarea::TextArea;

use crate::event::{spawn_event_reader, AppEvent};
use crate::navigator::Navigator;
use crate::state::{TabKind, TabState, TuiState};
use crate::ui;

/// Which panel currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Navigator,
    Content,
}

/// Content panel mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentMode {
    Command,
    Edit,
}

/// What kind of item a tab represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabKindLocal {
    Entity,
    Surface,
}

/// A tab open in the content panel.
pub struct Tab {
    pub title: String,
    pub id: String,
    pub kind: TabKindLocal,
    pub textarea: TextArea<'static>,
    pub dirty: bool,
    /// Content at last save — for dirty detection.
    saved_content: String,
}

impl Tab {
    fn new(title: String, id: String, kind: TabKindLocal, content: &str) -> Self {
        // Normalize escaped \n to real newlines
        let normalized = content.replace("\\n", "\n").replace("\r\n", "\n");
        let lines: Vec<String> = normalized.split('\n').map(|l| l.to_string()).collect();
        let saved = lines.join("\n");
        let mut textarea = TextArea::new(lines);
        textarea.set_cursor_style(
            ratatui::style::Style::default()
                .add_modifier(ratatui::style::Modifier::REVERSED),
        );
        textarea.set_cursor_line_style(
            ratatui::style::Style::default()
                .add_modifier(ratatui::style::Modifier::UNDERLINED),
        );
        Self {
            title,
            id,
            kind,
            textarea,
            dirty: false,
            saved_content: saved,
        }
    }

    fn content(&self) -> String {
        self.textarea.lines().join("\n")
    }

    fn check_dirty(&mut self) {
        self.dirty = self.content() != self.saved_content;
    }

    fn mark_saved(&mut self) {
        self.saved_content = self.content();
        self.dirty = false;
    }
}

/// Top-level application state.
pub struct App {
    pub workspace: PathBuf,
    pub focused: FocusedPanel,
    pub should_quit: bool,
    pub navigator: Navigator,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub content_mode: ContentMode,
    pub nav_width_pct: u16,
    /// Transient preview of the currently highlighted entity (not a tab).
    pub preview: Option<Tab>,
    /// ID of the entity currently being previewed.
    preview_id: Option<String>,
    pub show_help: bool,
    pub status_message: Option<String>,
    known_surface_ids: std::collections::HashSet<String>,
}

impl App {
    pub fn new(workspace: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let mut navigator = Navigator::new();
        let db_path = workspace.join("data/entities.db");

        let saved = TuiState::load(&workspace);

        for (entity_type, expanded) in &saved.navigator_expanded {
            navigator.set_expanded(entity_type, *expanded);
        }

        navigator.refresh(&db_path);

        let entity_store = clotho_store::data::entities::EntityStore::open(&db_path).ok();
        let surface_store = clotho_store::data::surfaces::SurfaceStore::open(&db_path).ok();
        let mut tabs = Vec::new();
        for tab_state in &saved.tabs {
            match tab_state.kind {
                TabKind::Entity => {
                    if let Some(ref store) = entity_store {
                        if let Ok(Some(entity)) = store.get(&tab_state.id) {
                            let content = if let Some(ref content_path) = entity.content_path {
                                let full_path = workspace.join("content").join(content_path);
                                std::fs::read_to_string(&full_path).unwrap_or_else(|_| "(no content)".to_string())
                            } else {
                                format_entity_details(&entity)
                            };
                            tabs.push(Tab::new(entity.title.clone(), entity.id.clone(), TabKindLocal::Entity, &content));
                        }
                    }
                }
                TabKind::Surface => {
                    if let Some(ref store) = surface_store {
                        if let Ok(Some(surface)) = store.get(&tab_state.id) {
                            tabs.push(Tab::new(surface.title.clone(), surface.id.clone(), TabKindLocal::Surface, &surface.content));
                        }
                    }
                }
            }
        }

        let mut known_surface_ids = std::collections::HashSet::new();
        if let Some(ref store) = surface_store {
            if let Ok(active) = store.list_active() {
                for s in &active {
                    known_surface_ids.insert(s.id.clone());
                }
            }
        }

        let active_tab = if tabs.is_empty() { 0 } else { saved.active_tab.min(tabs.len() - 1) };

        Ok(Self {
            workspace,
            focused: FocusedPanel::Navigator,
            should_quit: false,
            navigator,
            tabs,
            active_tab,
            content_mode: ContentMode::Command,
            nav_width_pct: 20,
            preview: None,
            preview_id: None,
            show_help: false,
            status_message: None,
            known_surface_ids,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut events = spawn_event_reader(Duration::from_secs(2));
        let mut redraw_interval = tokio::time::interval(Duration::from_millis(33));

        loop {
            terminal.draw(|frame| ui::render(frame, self))?;

            tokio::select! {
                maybe_event = events.recv() => {
                    if let Some(event) = maybe_event {
                        match event {
                            AppEvent::Key(key) => self.handle_key(key),
                            AppEvent::Mouse(mouse) => self.handle_mouse(mouse),
                            AppEvent::Resize(_, _) => {}
                            AppEvent::Tick => self.on_tick(),
                        }
                    }
                    while let Ok(event) = events.try_recv() {
                        match event {
                            AppEvent::Key(key) => self.handle_key(key),
                            AppEvent::Mouse(mouse) => self.handle_mouse(mouse),
                            AppEvent::Resize(_, _) => {}
                            AppEvent::Tick => self.on_tick(),
                        }
                    }
                }
                _ = redraw_interval.tick() => {}
            }

            if self.should_quit {
                break;
            }
        }

        self.save_state();

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), crossterm::event::DisableMouseCapture, LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                match self.focused {
                    FocusedPanel::Content => {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            for _ in 0..3 {
                                tab.textarea.move_cursor(ratatui_textarea::CursorMove::Up);
                            }
                        }
                    }
                    FocusedPanel::Navigator => {
                        for _ in 0..3 { self.navigator.cursor_up(); }
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                match self.focused {
                    FocusedPanel::Content => {
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            for _ in 0..3 {
                                tab.textarea.move_cursor(ratatui_textarea::CursorMove::Down);
                            }
                        }
                    }
                    FocusedPanel::Navigator => {
                        for _ in 0..3 { self.navigator.cursor_down(); }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if self.show_help {
            self.show_help = false;
            return;
        }

        // Global keybindings
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return;
            }
            // Esc: exit edit mode only
            (KeyCode::Esc, _) if self.focused == FocusedPanel::Content && self.content_mode == ContentMode::Edit => {
                self.content_mode = ContentMode::Command;
                self.status_message = None;
                return;
            }
            // Tab: switch panels (except in edit mode where it inserts)
            (KeyCode::Tab, _) if !(self.focused == FocusedPanel::Content && self.content_mode == ContentMode::Edit) => {
                self.cycle_focus();
                if self.focused == FocusedPanel::Content {
                    self.content_mode = ContentMode::Command;
                }
                return;
            }
            _ => {}
        }

        match self.focused {
            FocusedPanel::Navigator => self.handle_navigator_key(key),
            FocusedPanel::Content => self.handle_content_key(key),
        }
    }

    fn handle_navigator_key(&mut self, key: KeyEvent) {
        if self.navigator.searching {
            self.handle_navigator_search_key(key);
            return;
        }

        match key.code {
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('/') => self.navigator.start_search(),
            KeyCode::Char('a') => {
                self.navigator.show_archived = !self.navigator.show_archived;
                self.status_message = Some(
                    if self.navigator.show_archived { "Showing all".to_string() }
                    else { "Active only".to_string() }
                );
            }
            KeyCode::Char('<') | KeyCode::Char(',') => {
                if self.nav_width_pct > 10 { self.nav_width_pct -= 5; }
            }
            KeyCode::Char('>') | KeyCode::Char('.') => {
                if self.nav_width_pct < 50 { self.nav_width_pct += 5; }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.navigator.cursor_up();
                self.update_preview();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.navigator.cursor_down();
                self.update_preview();
            }
            KeyCode::Enter | KeyCode::Right => {
                if let Some((_, None)) = self.navigator.resolve_cursor() {
                    self.navigator.toggle_expand();
                } else if let Some((id, _title)) = self.navigator.selected_surface() {
                    let id = id.to_string();
                    self.open_surface_tab(&id);
                } else if let Some(entity) = self.navigator.selected_entity() {
                    self.open_entity_tab(entity.clone());
                }
            }
            KeyCode::Left => {
                if let Some((_, None)) = self.navigator.resolve_cursor() {
                    self.navigator.toggle_expand();
                }
            }
            _ => {}
        }
    }

    fn handle_navigator_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.navigator.stop_search();
                self.update_preview();
            }
            KeyCode::Enter => {
                if let Some(entity) = self.navigator.selected_search_entity() {
                    let entity = entity.clone();
                    self.navigator.stop_search();
                    self.open_entity_tab(entity);
                }
            }
            KeyCode::Up => { self.navigator.cursor_up(); self.update_preview(); }
            KeyCode::Down => { self.navigator.cursor_down(); self.update_preview(); }
            KeyCode::Backspace => { self.navigator.search_pop(); self.update_preview(); }
            KeyCode::Char(c) => { self.navigator.search_push(c); self.update_preview(); }
            _ => {}
        }
    }

    fn handle_content_key(&mut self, key: KeyEvent) {
        match self.content_mode {
            ContentMode::Command => self.handle_content_command_key(key),
            ContentMode::Edit => self.handle_content_edit_key(key),
        }
    }

    fn handle_content_command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Enter | KeyCode::Char('i') => {
                self.content_mode = ContentMode::Edit;
                self.status_message = Some("-- EDIT --".to_string());
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if self.active_tab > 0 { self.active_tab -= 1; }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if !self.tabs.is_empty() && self.active_tab < self.tabs.len() - 1 {
                    self.active_tab += 1;
                }
            }
            KeyCode::Char('w') => {
                if !self.tabs.is_empty() {
                    self.tabs.remove(self.active_tab);
                    if self.active_tab > 0 && self.active_tab >= self.tabs.len() {
                        self.active_tab = self.tabs.len() - 1;
                    }
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.textarea.move_cursor(ratatui_textarea::CursorMove::Down);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.textarea.move_cursor(ratatui_textarea::CursorMove::Up);
                }
            }
            KeyCode::PageUp => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    for _ in 0..20 { tab.textarea.move_cursor(ratatui_textarea::CursorMove::Up); }
                }
            }
            KeyCode::PageDown => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    for _ in 0..20 { tab.textarea.move_cursor(ratatui_textarea::CursorMove::Down); }
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.textarea.move_cursor(ratatui_textarea::CursorMove::Top);
                }
            }
            KeyCode::End | KeyCode::Char('G') => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    tab.textarea.move_cursor(ratatui_textarea::CursorMove::Bottom);
                }
            }
            KeyCode::Char('x') => {
                self.toggle_checkbox();
            }
            KeyCode::Char('s') => {
                self.save_active_tab();
                self.status_message = Some("Saved".to_string());
            }
            _ => {}
        }
    }

    fn handle_content_edit_key(&mut self, key: KeyEvent) {
        // Ctrl+S to save
        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.save_active_tab();
            self.status_message = Some("-- EDIT -- Saved".to_string());
            return;
        }

        // Pass everything else to the textarea
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.textarea.input(key);
            tab.check_dirty();
        }
    }

    fn toggle_checkbox(&mut self) {
        let Some(tab) = self.tabs.get_mut(self.active_tab) else { return };
        let (row, _) = tab.textarea.cursor();
        let lines = tab.textarea.lines();
        if row >= lines.len() { return; }

        let line = &lines[row];
        let new_line = if line.contains('☐') {
            Some(line.replacen('☐', "☑", 1))
        } else if line.contains('☑') {
            Some(line.replacen('☑', "☐", 1))
        } else if line.contains("[ ]") {
            Some(line.replacen("[ ]", "[x]", 1))
        } else if line.contains("[x]") || line.contains("[X]") {
            Some(line.replacen("[x]", "[ ]", 1).replacen("[X]", "[ ]", 1))
        } else {
            None
        };

        if let Some(new) = new_line {
            // Replace the line by selecting it and inserting the replacement
            let col = tab.textarea.cursor().1;
            tab.textarea.move_cursor(ratatui_textarea::CursorMove::Head);
            tab.textarea.delete_line_by_end();
            tab.textarea.insert_str(&new);
            // Try to restore column position
            tab.textarea.move_cursor(ratatui_textarea::CursorMove::Head);
            for _ in 0..col {
                tab.textarea.move_cursor(ratatui_textarea::CursorMove::Forward);
            }
            tab.check_dirty();
        }
    }

    fn save_active_tab(&mut self) {
        let Some(tab) = self.tabs.get_mut(self.active_tab) else { return };
        if !tab.dirty { return; }

        let db_path = self.workspace.join("data/entities.db");
        let content = tab.content();

        match tab.kind {
            TabKindLocal::Surface => {
                if let Ok(store) = clotho_store::data::surfaces::SurfaceStore::open(&db_path) {
                    if store.update_content(&tab.id, &content).is_ok() {
                        tab.mark_saved();
                    }
                }
            }
            TabKindLocal::Entity => {
                if let Ok(store) = clotho_store::data::entities::EntityStore::open(&db_path) {
                    if let Ok(Some(entity)) = store.get(&tab.id) {
                        if let Some(ref content_path) = entity.content_path {
                            let full_path = self.workspace.join("content").join(content_path);
                            if std::fs::write(&full_path, &content).is_ok() {
                                tab.mark_saved();
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_preview(&mut self) {
        // Check for surface first
        if !self.navigator.searching {
            if let Some((id, _title)) = self.navigator.selected_surface() {
                let id = id.to_string();
                if self.preview_id.as_deref() == Some(&id) { return; }

                let db_path = self.workspace.join("data/entities.db");
                if let Ok(store) = clotho_store::data::surfaces::SurfaceStore::open(&db_path) {
                    if let Ok(Some(surface)) = store.get(&id) {
                        self.preview_id = Some(surface.id.clone());
                        self.preview = Some(Tab::new(surface.title.clone(), surface.id.clone(), TabKindLocal::Surface, &surface.content));
                        return;
                    }
                }
            }
        }

        // Get the currently highlighted entity from navigator
        let entity = if self.navigator.searching {
            self.navigator.selected_search_entity().cloned()
        } else {
            self.navigator.selected_entity().cloned()
        };

        let Some(entity) = entity else {
            self.preview = None;
            self.preview_id = None;
            return;
        };

        if self.preview_id.as_deref() == Some(&entity.id) { return; }

        let body = if let Some(ref content_path) = entity.content_path {
            let full_path = self.workspace.join("content").join(content_path);
            std::fs::read_to_string(&full_path).unwrap_or_else(|_| "(no content)".to_string())
        } else {
            format_entity_details(&entity)
        };

        let relations = self.load_relations_header(&entity);
        let content = if relations.is_empty() { body } else { format!("{}\n---\n\n{}", relations, body) };

        self.preview_id = Some(entity.id.clone());
        self.preview = Some(Tab::new(entity.title.clone(), entity.id.clone(), TabKindLocal::Entity, &content));
    }

    fn open_entity_tab(&mut self, entity: clotho_store::data::entities::EntityRow) {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == entity.id) {
            self.active_tab = idx;
            return;
        }

        let body = if let Some(ref content_path) = entity.content_path {
            let full_path = self.workspace.join("content").join(content_path);
            std::fs::read_to_string(&full_path).unwrap_or_else(|_| "(no content)".to_string())
        } else {
            format_entity_details(&entity)
        };

        // Build relations header
        let relations = self.load_relations_header(&entity);
        let content = if relations.is_empty() {
            body
        } else {
            format!("{}\n---\n\n{}", relations, body)
        };

        self.tabs.push(Tab::new(entity.title.clone(), entity.id.clone(), TabKindLocal::Entity, &content));
        self.active_tab = self.tabs.len() - 1;
        self.focused = FocusedPanel::Content;
    }

    fn open_surface_tab(&mut self, surface_id: &str) {
        // Don't open duplicate
        if let Some(idx) = self.tabs.iter().position(|t| t.id == surface_id) {
            self.active_tab = idx;
            self.focused = FocusedPanel::Content;
            return;
        }

        let db_path = self.workspace.join("data/entities.db");
        if let Ok(store) = clotho_store::data::surfaces::SurfaceStore::open(&db_path) {
            if let Ok(Some(surface)) = store.get(surface_id) {
                self.tabs.push(Tab::new(
                    surface.title.clone(),
                    surface.id.clone(),
                    TabKindLocal::Surface,
                    &surface.content,
                ));
                self.active_tab = self.tabs.len() - 1;
                self.focused = FocusedPanel::Content;
            }
        }
    }

    fn load_relations_header(&self, entity: &clotho_store::data::entities::EntityRow) -> String {
        let graph_path = self.workspace.join("graph/relations.db");
        let db_path = self.workspace.join("data/entities.db");

        let graph = match clotho_core::graph::GraphStore::open(&graph_path) {
            Ok(g) => g,
            Err(_) => return String::new(),
        };
        let store = match clotho_store::data::entities::EntityStore::open(&db_path) {
            Ok(s) => s,
            Err(_) => return String::new(),
        };

        let entity_id = match uuid::Uuid::parse_str(&entity.id) {
            Ok(u) => clotho_core::domain::types::EntityId::from(u),
            Err(_) => return String::new(),
        };

        // Get outgoing and incoming edges
        let outgoing = graph.get_edges_from(&entity_id).unwrap_or_default();
        let incoming = graph.get_edges_to(&entity_id).unwrap_or_default();

        if outgoing.is_empty() && incoming.is_empty() {
            return String::new();
        }

        let mut lines = Vec::new();
        lines.push(format!("## {} ({})", entity.title, entity.entity_type));
        lines.push(String::new());

        // Group outgoing by relation type
        let mut out_grouped: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
        for edge in &outgoing {
            let target_title = store.get(&edge.target_id.to_string())
                .ok().flatten()
                .map(|r| format!("{} ({})", r.title, r.entity_type))
                .unwrap_or_else(|| edge.target_id.to_string()[..8].to_string());
            let rel = format!("{:?}", edge.relation_type);
            out_grouped.entry(rel).or_default().push(target_title);
        }

        // Group incoming by relation type
        let mut in_grouped: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
        for edge in &incoming {
            let source_title = store.get(&edge.source_id.to_string())
                .ok().flatten()
                .map(|r| format!("{} ({})", r.title, r.entity_type))
                .unwrap_or_else(|| edge.source_id.to_string()[..8].to_string());
            let rel = format!("{:?}", edge.relation_type);
            in_grouped.entry(rel).or_default().push(source_title);
        }

        if !out_grouped.is_empty() {
            for (rel, targets) in &out_grouped {
                lines.push(format!("{}: {}", rel, targets.join(", ")));
            }
        }

        if !in_grouped.is_empty() {
            for (rel, sources) in &in_grouped {
                lines.push(format!("{} (from): {}", rel, sources.join(", ")));
            }
        }

        lines.join("\n")
    }

    fn cycle_focus(&mut self) {
        self.focused = match self.focused {
            FocusedPanel::Navigator => FocusedPanel::Content,
            FocusedPanel::Content => FocusedPanel::Navigator,
        };
    }

    fn on_tick(&mut self) {
        let db_path = self.workspace.join("data/entities.db");
        self.navigator.refresh(&db_path);

        if let Ok(store) = clotho_store::data::surfaces::SurfaceStore::open(&db_path) {
            if let Ok(active) = store.list_active() {
                for surface in active {
                    if !self.known_surface_ids.contains(&surface.id) {
                        self.known_surface_ids.insert(surface.id.clone());
                        if !self.tabs.iter().any(|t| t.id == surface.id) {
                            self.tabs.push(Tab::new(
                                surface.title.clone(),
                                surface.id.clone(),
                                TabKindLocal::Surface,
                                &surface.content,
                            ));
                            self.active_tab = self.tabs.len() - 1;
                        }
                    } else {
                        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == surface.id) {
                            if !tab.dirty && tab.content() != surface.content {
                                let (row, col) = tab.textarea.cursor();
                                *tab = Tab::new(surface.title.clone(), surface.id.clone(), TabKindLocal::Surface, &surface.content);
                                // Restore cursor
                                tab.textarea.move_cursor(ratatui_textarea::CursorMove::Top);
                                for _ in 0..row {
                                    tab.textarea.move_cursor(ratatui_textarea::CursorMove::Down);
                                }
                                for _ in 0..col {
                                    tab.textarea.move_cursor(ratatui_textarea::CursorMove::Forward);
                                }
                            } else {
                                tab.title = surface.title.clone();
                            }
                        }
                    }
                }
            }
        }

        self.save_state();
    }

    fn save_state(&self) {
        let tabs: Vec<TabState> = self.tabs.iter().map(|t| TabState {
            kind: match t.kind {
                TabKindLocal::Entity => TabKind::Entity,
                TabKindLocal::Surface => TabKind::Surface,
            },
            id: t.id.clone(),
        }).collect();

        let mut navigator_expanded = HashMap::new();
        for section in &self.navigator.sections {
            navigator_expanded.insert(section.title.clone(), section.expanded);
            for item in &section.items {
                if let crate::navigator::NavItem::SubSection { title, expanded, .. } = item {
                    navigator_expanded.insert(title.clone(), *expanded);
                }
            }
        }

        let state = TuiState { tabs, active_tab: self.active_tab, navigator_expanded };
        state.save(&self.workspace);
    }
}

fn format_entity_details(entity: &clotho_store::data::entities::EntityRow) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {}", entity.title));
    lines.push(String::new());
    lines.push(format!("Type: {}", entity.entity_type));
    lines.push(format!("ID:   {}", entity.id));
    if let Some(ref status) = entity.status { lines.push(format!("Status: {}", status)); }
    if let Some(ref state) = entity.task_state { lines.push(format!("State: {}", state)); }
    lines.push(format!("Created: {}", entity.created_at));
    lines.push(format!("Updated: {}", entity.updated_at));
    if let Some(ref metadata) = entity.metadata {
        lines.push(String::new());
        lines.push("Metadata:".to_string());
        lines.push(metadata.clone());
    }
    lines.join("\n")
}
