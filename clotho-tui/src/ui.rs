use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use tui_term::widget::PseudoTerminal;
use unicode_width::UnicodeWidthChar;

use crate::app::{App, ContentMode, FocusedPanel};

/// Render the full TUI layout.
pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Main area + status bar
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // main area
            Constraint::Length(1),   // status bar
        ])
        .split(size);

    let main_area = outer[0];
    let status_area = outer[1];

    // Top-level horizontal split: navigator (left) | main area (right)
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(app.nav_width_pct),
            Constraint::Percentage(100 - app.nav_width_pct),
        ])
        .split(main_area);

    // Right side vertical split: content (top) | chat (bottom)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // content area
            Constraint::Percentage(50), // chat terminal
        ])
        .split(horizontal[1]);

    render_navigator(frame, app, horizontal[0]);
    render_content(frame, app, vertical[0]);
    render_chat(frame, app, vertical[1]);
    render_status_bar(frame, app, status_area);

    // Help overlay on top of everything
    if app.show_help {
        render_help_overlay(frame, size);
    }
}

fn panel_border_style(app: &App, panel: FocusedPanel) -> Style {
    if app.focused == panel {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(80, 80, 90))
    }
}

fn panel_border_type(app: &App, panel: FocusedPanel) -> ratatui::widgets::BorderType {
    if app.focused == panel {
        ratatui::widgets::BorderType::Thick
    } else {
        ratatui::widgets::BorderType::Plain
    }
}

fn render_navigator(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" Entities ")
        .borders(Borders::ALL)
        .border_type(panel_border_type(app, FocusedPanel::Navigator))
        .border_style(panel_border_style(app, FocusedPanel::Navigator));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Compute available height for the navigator list
    let height = inner.height as usize;
    app.navigator.adjust_scroll(height);
    let lines = app.navigator.visible_lines(height);

    let text_lines: Vec<Line> = lines
        .iter()
        .map(|(text, is_header, is_cursor)| {
            let style = if *is_cursor {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else if *is_header {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(text.clone(), style))
        })
        .collect();

    let paragraph = Paragraph::new(text_lines);
    frame.render_widget(paragraph, inner);
}

fn render_content(frame: &mut Frame, app: &mut App, area: Rect) {
    let mode_label = match (app.focused, app.content_mode) {
        (FocusedPanel::Content, ContentMode::Edit) => " Content [EDIT] ",
        (FocusedPanel::Content, ContentMode::Command) => " Content [CMD] ",
        _ => " Content ",
    };
    let block = Block::default()
        .title(mode_label)
        .borders(Borders::ALL)
        .border_type(panel_border_type(app, FocusedPanel::Content))
        .border_style(panel_border_style(app, FocusedPanel::Content));

    if app.tabs.is_empty() {
        let content = Paragraph::new("No tabs open. Select an entity from the navigator.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(content, area);
        return;
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner into tab bar + content
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // tab bar
            Constraint::Min(0),   // content
        ])
        .split(inner);

    // Tab bar
    let tab_titles: Vec<Line> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let dirty = if t.editor.dirty { "● " } else { "" };
            let title = if t.title.len() > 18 {
                format!("{}{}…", dirty, &t.title[..17])
            } else {
                format!("{}{}", dirty, t.title)
            };
            if i == app.active_tab {
                Line::from(title)
            } else {
                Line::from(title)
            }
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .select(app.active_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .divider("│");

    frame.render_widget(tabs, content_layout[0]);

    // Tab content with editor
    if let Some(tab) = app.tabs.get_mut(app.active_tab) {
        let content_area = content_layout[1];
        let width = content_area.width as usize;
        let viewport_height = content_area.height as usize;

        if width == 0 || viewport_height == 0 {
            return;
        }

        // Store viewport height for page up/down
        app.content_viewport_height = viewport_height;

        // Manually wrap lines to fit the content area width using display width
        let mut visual_lines: Vec<(usize, String)> = Vec::new(); // (editor_row, text)
        for (row_idx, line) in tab.editor.lines.iter().enumerate() {
            if line.is_empty() {
                visual_lines.push((row_idx, String::new()));
            } else {
                // Wrap by display width, not character count
                let mut current = String::new();
                let mut current_width: usize = 0;
                for ch in line.chars() {
                    let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);
                    if current_width + ch_width > width {
                        visual_lines.push((row_idx, current));
                        current = String::new();
                        current_width = 0;
                    }
                    current.push(ch);
                    current_width += ch_width;
                }
                visual_lines.push((row_idx, current));
            }
        }

        // Find which visual line the cursor is on
        let mut cursor_visual_row = 0;
        let mut cursor_visual_col = tab.editor.cursor_col;
        {
            let mut vrow = 0;
            for (row_idx, _) in &visual_lines {
                if *row_idx == tab.editor.cursor_row {
                    if cursor_visual_col < width {
                        cursor_visual_row = vrow;
                        break;
                    }
                    cursor_visual_col -= width;
                }
                vrow += 1;
            }
            // If we exhausted visual lines for the cursor row
            if cursor_visual_col >= width {
                cursor_visual_col = cursor_visual_col % width;
            }
        }

        // Adjust scroll based on visual cursor position
        tab.editor.adjust_scroll(viewport_height);
        let mut scroll = tab.editor.scroll_offset;
        // Convert scroll from editor rows to visual rows
        let mut visual_scroll = 0;
        {
            let mut editor_rows_seen = 0;
            for (i, (row_idx, _)) in visual_lines.iter().enumerate() {
                if i > 0 && *row_idx != visual_lines[i - 1].0 {
                    editor_rows_seen = *row_idx;
                }
                if editor_rows_seen >= scroll {
                    visual_scroll = i;
                    break;
                }
            }
        }

        // Keep cursor visible
        if cursor_visual_row < visual_scroll {
            visual_scroll = cursor_visual_row;
        }
        if cursor_visual_row >= visual_scroll + viewport_height {
            visual_scroll = cursor_visual_row - viewport_height + 1;
        }

        let is_focused = app.focused == FocusedPanel::Content;

        // Render visible visual lines
        let visible: Vec<Line> = visual_lines
            .iter()
            .skip(visual_scroll)
            .take(viewport_height)
            .map(|(_, text)| {
                Line::from(Span::styled(text.clone(), Style::default().fg(Color::White)))
            })
            .collect();

        let paragraph = Paragraph::new(visible);
        frame.render_widget(paragraph, content_area);

        // Overlay cursor
        if is_focused {
            let cursor_screen_row = cursor_visual_row.saturating_sub(visual_scroll);
            let cursor_x = content_area.x + cursor_visual_col as u16;
            let cursor_y = content_area.y + cursor_screen_row as u16;

            if cursor_x < content_area.x + content_area.width
                && cursor_y < content_area.y + content_area.height
            {
                let ch = tab.editor.lines
                    .get(tab.editor.cursor_row)
                    .and_then(|line| line.chars().nth(tab.editor.cursor_col))
                    .unwrap_or(' ');

                let cursor_span = Span::styled(
                    ch.to_string(),
                    Style::default().bg(Color::White).fg(Color::Black),
                );
                frame.buffer_mut().set_span(cursor_x, cursor_y, &cursor_span, 1);
            }
        }
    }
}


fn render_chat(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" Chat ")
        .borders(Borders::ALL)
        .border_type(panel_border_type(app, FocusedPanel::Chat))
        .border_style(panel_border_style(app, FocusedPanel::Chat));

    // Resize PTY to match the inner area of the chat panel
    let inner = block.inner(area);
    if let Some(ref mut pty) = app.pty {
        pty.resize(inner.height, inner.width);
    }

    if let Some(ref pty) = app.pty {
        if let Ok(parser) = pty.parser.read() {
            let pseudo_term = PseudoTerminal::new(parser.screen()).block(block);
            frame.render_widget(pseudo_term, area);
        } else {
            let content = Paragraph::new("PTY lock contention — retrying...")
                .block(block);
            frame.render_widget(content, area);
        }
    } else {
        let content = Paragraph::new("claude not available. Install claude CLI to use chat.")
            .block(block);
        frame.render_widget(content, area);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let panel_name = match app.focused {
        FocusedPanel::Navigator => "NAV",
        FocusedPanel::Content => match app.content_mode {
            ContentMode::Command => "CONTENT:CMD",
            ContentMode::Edit => "CONTENT:EDIT",
        },
        FocusedPanel::Chat => "CHAT",
    };

    let dirty = app.tabs.get(app.active_tab).map_or(false, |t| t.editor.dirty);
    let dirty_indicator = if dirty { " [modified]" } else { "" };

    let left = format!(" {} {}{}", panel_name,
        app.status_message.as_deref().unwrap_or(""),
        dirty_indicator,
    );

    let right = format!("Ctrl+Tab: switch panel | ?: help ");

    let width = area.width as usize;
    let padding = width.saturating_sub(left.len() + right.len());
    let bar_text = format!("{}{:pad$}{}", left, "", right, pad = padding);

    let bar = Paragraph::new(bar_text)
        .style(Style::default().bg(Color::Rgb(40, 44, 52)).fg(Color::Rgb(171, 178, 191)));
    frame.render_widget(bar, area);
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Center a box in the middle of the screen
    let width = 60u16.min(area.width.saturating_sub(4));
    let height = 28u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let overlay_area = Rect::new(x, y, width, height);

    // Clear the area
    let block = Block::default()
        .title(" Keybindings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(Span::styled("GLOBAL", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Ctrl+Tab     Switch panel focus"),
        Line::from("  Ctrl+Q       Quit"),
        Line::from("  ?            Show this help"),
        Line::from(""),
        Line::from(Span::styled("ENTITIES PANEL", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  j/k or Up/Dn  Move cursor"),
        Line::from("  Enter/Right   Open entity / expand group"),
        Line::from("  Left          Collapse group"),
        Line::from("  < / >         Resize panel"),
        Line::from(""),
        Line::from(Span::styled("CONTENT - COMMAND MODE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  h/l or L/R    Previous / next tab"),
        Line::from("  j/k or Up/Dn  Scroll content"),
        Line::from("  PgUp/PgDn     Page scroll"),
        Line::from("  Home/g        Top of document"),
        Line::from("  End/G         Bottom of document"),
        Line::from("  i or Enter    Enter edit mode"),
        Line::from("  w             Close tab"),
        Line::from("  x             Toggle checkbox"),
        Line::from("  s             Save"),
        Line::from(""),
        Line::from(Span::styled("CONTENT - EDIT MODE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Esc           Exit to command mode"),
        Line::from("  Ctrl+S        Save"),
        Line::from("  Arrows        Move cursor"),
        Line::from("  Home/End      Start/end of line"),
        Line::from("  PgUp/PgDn     Page scroll"),
        Line::from("  Type          Insert text"),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, overlay_area);
}
