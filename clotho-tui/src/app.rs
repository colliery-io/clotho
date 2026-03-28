use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::editor::Editor;
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
    pub editor: Editor,
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
    pub content_viewport_height: usize,
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
                                format_entity_details_static(&entity)
                            };
                            tabs.push(Tab {
                                title: entity.title.clone(),
                                id: entity.id.clone(),
                                kind: TabKindLocal::Entity,
                                editor: Editor::new(&content),
                            });
                        }
                    }
                }
                TabKind::Surface => {
                    if let Some(ref store) = surface_store {
                        if let Ok(Some(surface)) = store.get(&tab_state.id) {
                            tabs.push(Tab {
                                title: surface.title.clone(),
                                id: surface.id.clone(),
                                kind: TabKindLocal::Surface,
                                editor: Editor::new(&surface.content),
                            });
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
            content_viewport_height: 20,
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
                            for _ in 0..3 { tab.editor.move_up(); }
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
                            for _ in 0..3 { tab.editor.move_down(); }
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

        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return;
            }
            (KeyCode::Tab, KeyModifiers::CONTROL) | (KeyCode::Tab, KeyModifiers::NONE) if self.focused == FocusedPanel::Navigator => {
                self.cycle_focus();
                return;
            }
            (KeyCode::Tab, KeyModifiers::CONTROL) => {
                self.cycle_focus();
                if self.focused == FocusedPanel::Content {
                    self.content_mode = ContentMode::Command;
                }
                return;
            }
            (KeyCode::BackTab, _) => {
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
        match key.code {
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('<') | KeyCode::Char(',') => {
                if self.nav_width_pct > 10 { self.nav_width_pct -= 5; }
            }
            KeyCode::Char('>') | KeyCode::Char('.') => {
                if self.nav_width_pct < 50 { self.nav_width_pct += 5; }
            }
            KeyCode::Up | KeyCode::Char('k') => self.navigator.cursor_up(),
            KeyCode::Down | KeyCode::Char('j') => self.navigator.cursor_down(),
            KeyCode::Enter | KeyCode::Right => {
                if let Some((_, None)) = self.navigator.resolve_cursor() {
                    self.navigator.toggle_expand();
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
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_down(); }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_up(); }
            }
            KeyCode::PageUp => {
                let h = self.content_viewport_height;
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.page_up(h); }
            }
            KeyCode::PageDown => {
                let h = self.content_viewport_height;
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.page_down(h); }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_to_start(); }
            }
            KeyCode::End | KeyCode::Char('G') => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_to_end(); }
            }
            KeyCode::Char('x') => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.toggle_checkbox(); }
            }
            KeyCode::Char('s') => {
                self.save_active_tab();
                self.status_message = Some("Saved".to_string());
            }
            _ => {}
        }
    }

    fn handle_content_edit_key(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => {
                self.content_mode = ContentMode::Command;
                self.status_message = None;
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.save_active_tab();
                self.status_message = Some("-- EDIT -- Saved".to_string());
            }
            (KeyCode::Up, _) => { if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_up(); } }
            (KeyCode::Down, _) => { if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_down(); } }
            (KeyCode::Left, _) => { if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_left(); } }
            (KeyCode::Right, _) => { if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_right(); } }
            (KeyCode::Home, _) => { if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_home(); } }
            (KeyCode::End, _) => { if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.move_end(); } }
            (KeyCode::PageUp, _) => {
                let h = self.content_viewport_height;
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.page_up(h); }
            }
            (KeyCode::PageDown, _) => {
                let h = self.content_viewport_height;
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.page_down(h); }
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.insert_char(c); }
            }
            (KeyCode::Enter, _) => { if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.insert_newline(); } }
            (KeyCode::Backspace, _) => { if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.backspace(); } }
            (KeyCode::Delete, _) => { if let Some(tab) = self.tabs.get_mut(self.active_tab) { tab.editor.delete(); } }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    for _ in 0..4 { tab.editor.insert_char(' '); }
                }
            }
            _ => {}
        }
    }

    fn save_active_tab(&mut self) {
        let Some(tab) = self.tabs.get_mut(self.active_tab) else { return };
        if !tab.editor.dirty { return; }

        let db_path = self.workspace.join("data/entities.db");
        let content = tab.editor.content();

        match tab.kind {
            TabKindLocal::Surface => {
                if let Ok(store) = clotho_store::data::surfaces::SurfaceStore::open(&db_path) {
                    if store.update_content(&tab.id, &content).is_ok() {
                        tab.editor.dirty = false;
                    }
                }
            }
            TabKindLocal::Entity => {
                if let Ok(store) = clotho_store::data::entities::EntityStore::open(&db_path) {
                    if let Ok(Some(entity)) = store.get(&tab.id) {
                        if let Some(ref content_path) = entity.content_path {
                            let full_path = self.workspace.join("content").join(content_path);
                            if std::fs::write(&full_path, &content).is_ok() {
                                tab.editor.dirty = false;
                            }
                        }
                    }
                }
            }
        }
    }

    fn open_entity_tab(&mut self, entity: clotho_store::data::entities::EntityRow) {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == entity.id) {
            self.active_tab = idx;
            return;
        }

        let content = if let Some(ref content_path) = entity.content_path {
            let full_path = self.workspace.join("content").join(content_path);
            std::fs::read_to_string(&full_path).unwrap_or_else(|_| "(no content)".to_string())
        } else {
            format_entity_details_static(&entity)
        };

        self.tabs.push(Tab {
            title: entity.title.clone(),
            id: entity.id.clone(),
            kind: TabKindLocal::Entity,
            editor: Editor::new(&content),
        });
        self.active_tab = self.tabs.len() - 1;
        self.focused = FocusedPanel::Content;
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
                            self.tabs.push(Tab {
                                title: surface.title.clone(),
                                id: surface.id.clone(),
                                kind: TabKindLocal::Surface,
                                editor: Editor::new(&surface.content),
                            });
                            self.active_tab = self.tabs.len() - 1;
                        }
                    } else {
                        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == surface.id) {
                            if !tab.editor.dirty {
                                let new_content = surface.content.clone();
                                if tab.editor.content() != new_content {
                                    let row = tab.editor.cursor_row;
                                    let col = tab.editor.cursor_col;
                                    let scroll = tab.editor.scroll_offset;
                                    tab.editor = Editor::new(&new_content);
                                    tab.editor.cursor_row = row.min(tab.editor.lines.len().saturating_sub(1));
                                    tab.editor.cursor_col = col;
                                    tab.editor.scroll_offset = scroll;
                                }
                            }
                            tab.title = surface.title.clone();
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

        let navigator_expanded = self.navigator.groups.iter()
            .map(|g| (g.entity_type.clone(), g.expanded))
            .collect();

        let state = TuiState { tabs, active_tab: self.active_tab, navigator_expanded };
        state.save(&self.workspace);
    }
}

fn format_entity_details_static(entity: &clotho_store::data::entities::EntityRow) -> String {
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
