use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, ContentMode, FocusedPanel};

pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    let main_area = outer[0];
    let status_area = outer[1];

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(app.nav_width_pct),
            Constraint::Percentage(100 - app.nav_width_pct),
        ])
        .split(main_area);

    render_navigator(frame, app, horizontal[0]);
    render_content(frame, app, horizontal[1]);
    render_status_bar(frame, app, status_area);

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
    let title = if app.navigator.searching {
        " Entities [SEARCH] "
    } else {
        " Entities "
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(panel_border_type(app, FocusedPanel::Navigator))
        .border_style(panel_border_style(app, FocusedPanel::Navigator));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.navigator.searching {
        // Split inner: search bar (top) + results (below)
        let nav_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(inner);

        // Search bar
        let search_text = format!("/{}", app.navigator.search_query);
        let search_bar = Paragraph::new(search_text)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(search_bar, nav_layout[0]);

        // Search results
        let height = nav_layout[1].height as usize;
        app.navigator.adjust_scroll(height);
        let lines = app.navigator.search_lines(height);

        let text_lines: Vec<Line> = lines
            .iter()
            .map(|(text, _is_header, is_cursor)| {
                let style = if *is_cursor {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(text.clone(), style))
            })
            .collect();

        let paragraph = Paragraph::new(text_lines);
        frame.render_widget(paragraph, nav_layout[1]);
    } else {
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

    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    // Tab bar
    let tab_titles: Vec<Line> = app.tabs.iter().map(|t| {
        let dirty = if t.dirty { "● " } else { "" };
        let title = if t.title.len() > 18 {
            format!("{}{}…", dirty, &t.title[..17])
        } else {
            format!("{}{}", dirty, t.title)
        };
        Line::from(title)
    }).collect();

    let tabs = Tabs::new(tab_titles)
        .select(app.active_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .divider("│");

    frame.render_widget(tabs, content_layout[0]);

    // Render the textarea widget
    if let Some(tab) = app.tabs.get_mut(app.active_tab) {
        let is_focused = app.focused == FocusedPanel::Content;
        let is_editing = is_focused && app.content_mode == ContentMode::Edit;

        // Show cursor only when editing
        if is_editing {
            tab.textarea.set_cursor_style(
                Style::default().add_modifier(Modifier::REVERSED),
            );
        } else {
            tab.textarea.set_cursor_style(Style::default());
        }

        frame.render_widget(&tab.textarea, content_layout[1]);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let panel_name = match app.focused {
        FocusedPanel::Navigator => "NAV",
        FocusedPanel::Content => match app.content_mode {
            ContentMode::Command => "CONTENT:CMD",
            ContentMode::Edit => "CONTENT:EDIT",
        },
    };

    let dirty = app.tabs.get(app.active_tab).map_or(false, |t| t.dirty);
    let dirty_indicator = if dirty { " [modified]" } else { "" };

    let left = format!(" {} {}{}", panel_name,
        app.status_message.as_deref().unwrap_or(""),
        dirty_indicator,
    );

    let right = "Tab: switch panel | ?: help ".to_string();

    let width = area.width as usize;
    let padding = width.saturating_sub(left.len() + right.len());
    let bar_text = format!("{}{:pad$}{}", left, "", right, pad = padding);

    let bar = Paragraph::new(bar_text)
        .style(Style::default().bg(Color::Rgb(40, 44, 52)).fg(Color::Rgb(171, 178, 191)));
    frame.render_widget(bar, area);
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let width = 55u16.min(area.width.saturating_sub(4));
    let height = 24u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let overlay_area = Rect::new(x, y, width, height);

    let block = Block::default()
        .title(" Keybindings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(Span::styled("GLOBAL", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Tab            Switch panel focus"),
        Line::from("  Ctrl+C/Q       Quit"),
        Line::from("  ?              Show this help"),
        Line::from(""),
        Line::from(Span::styled("ENTITIES PANEL", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  j/k or Up/Dn   Move cursor"),
        Line::from("  Enter/Right    Open entity / expand"),
        Line::from("  Left           Collapse group"),
        Line::from("  /              Search entities"),
        Line::from("  a              Toggle show archived"),
        Line::from("  < / >          Resize panel"),
        Line::from(""),
        Line::from(Span::styled("CONTENT - COMMAND MODE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  h/l or L/R     Previous / next tab"),
        Line::from("  j/k or Up/Dn   Scroll content"),
        Line::from("  PgUp/PgDn      Page scroll"),
        Line::from("  Home/g  End/G  Top / bottom"),
        Line::from("  i or Enter     Enter edit mode"),
        Line::from("  w  Close tab   s  Save   x  Checkbox"),
        Line::from(""),
        Line::from(Span::styled("CONTENT - EDIT MODE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Esc            Exit to command mode"),
        Line::from("  Ctrl+S         Save"),
        Line::from("  Full editing: undo/redo, select, copy/paste"),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .style(Style::default().bg(Color::Black));
    frame.render_widget(paragraph, overlay_area);
}
