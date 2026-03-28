/// A simple text editor buffer with cursor support.
#[derive(Debug, Clone)]
pub struct Editor {
    /// Lines of text.
    pub lines: Vec<String>,
    /// Cursor row (0-indexed line number).
    pub cursor_row: usize,
    /// Cursor column (0-indexed byte offset within the line).
    pub cursor_col: usize,
    /// Scroll offset (first visible line).
    pub scroll_offset: usize,
    /// Whether the content has been modified since last save.
    pub dirty: bool,
}

impl Editor {
    pub fn new(content: &str) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            // Handle both real newlines and literal \n escape sequences
            let normalized = content
                .replace("\\n", "\n")
                .replace("\r\n", "\n");
            normalized.split('\n').map(|l| l.to_string()).collect()
        };
        Self {
            lines,
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            dirty: false,
        }
    }

    /// Get the full content as a single string.
    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.ensure_cursor_valid();
        let line = &mut self.lines[self.cursor_row];
        // Find the byte index for the cursor column (handle UTF-8)
        let byte_idx = char_col_to_byte(line, self.cursor_col);
        line.insert(byte_idx, c);
        self.cursor_col += 1;
        self.dirty = true;
    }

    /// Insert a newline at cursor (split the current line).
    pub fn insert_newline(&mut self) {
        self.ensure_cursor_valid();
        let byte_idx = char_col_to_byte(&self.lines[self.cursor_row], self.cursor_col);
        let rest = self.lines[self.cursor_row][byte_idx..].to_string();
        self.lines[self.cursor_row].truncate(byte_idx);
        self.cursor_row += 1;
        self.lines.insert(self.cursor_row, rest);
        self.cursor_col = 0;
        self.dirty = true;
    }

    /// Delete the character before the cursor (backspace).
    pub fn backspace(&mut self) {
        self.ensure_cursor_valid();
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            let byte_idx = char_col_to_byte(line, self.cursor_col);
            let prev_byte_idx = char_col_to_byte(line, self.cursor_col - 1);
            line.drain(prev_byte_idx..byte_idx);
            self.cursor_col -= 1;
            self.dirty = true;
        } else if self.cursor_row > 0 {
            // Merge with previous line
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = char_len(&self.lines[self.cursor_row]);
            self.lines[self.cursor_row].push_str(&current);
            self.dirty = true;
        }
    }

    /// Delete the character at the cursor (delete key).
    pub fn delete(&mut self) {
        self.ensure_cursor_valid();
        let line_char_len = char_len(&self.lines[self.cursor_row]);
        if self.cursor_col < line_char_len {
            let line = &mut self.lines[self.cursor_row];
            let byte_idx = char_col_to_byte(line, self.cursor_col);
            let next_byte_idx = char_col_to_byte(line, self.cursor_col + 1);
            line.drain(byte_idx..next_byte_idx);
            self.dirty = true;
        } else if self.cursor_row + 1 < self.lines.len() {
            // Merge next line into current
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
            self.dirty = true;
        }
    }

    /// Move cursor up.
    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_cursor_col();
        }
    }

    /// Move cursor down.
    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.clamp_cursor_col();
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = char_len(&self.lines[self.cursor_row]);
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        let line_len = char_len(&self.lines[self.cursor_row]);
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor to start of line.
    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to end of line.
    pub fn move_end(&mut self) {
        self.cursor_col = char_len(&self.lines[self.cursor_row]);
    }

    /// Move cursor up by a page (viewport_height lines).
    pub fn page_up(&mut self, viewport_height: usize) {
        self.cursor_row = self.cursor_row.saturating_sub(viewport_height);
        self.clamp_cursor_col();
    }

    /// Move cursor down by a page (viewport_height lines).
    pub fn page_down(&mut self, viewport_height: usize) {
        self.cursor_row = (self.cursor_row + viewport_height).min(self.lines.len().saturating_sub(1));
        self.clamp_cursor_col();
    }

    /// Move cursor to the start of the document.
    pub fn move_to_start(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Move cursor to the end of the document.
    pub fn move_to_end(&mut self) {
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = char_len(&self.lines[self.cursor_row]);
    }

    /// Toggle checkbox on the current line.
    /// Converts ☐ → ☑ and ☑ → ☐, or [ ] → [x] and [x] → [ ].
    pub fn toggle_checkbox(&mut self) {
        self.ensure_cursor_valid();
        let line = &self.lines[self.cursor_row];

        let new_line = if line.contains('☐') {
            Some(line.replacen('☐', "☑", 1))
        } else if line.contains('☑') {
            Some(line.replacen('☑', "☐", 1))
        } else if line.contains("[ ]") {
            Some(line.replacen("[ ]", "[x]", 1))
        } else if line.contains("[x]") || line.contains("[X]") {
            let l = line.replacen("[x]", "[ ]", 1);
            Some(l.replacen("[X]", "[ ]", 1))
        } else {
            None
        };

        if let Some(new) = new_line {
            self.lines[self.cursor_row] = new;
            self.dirty = true;
        }
    }

    /// Adjust scroll to keep cursor visible.
    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        }
        if self.cursor_row >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor_row - viewport_height + 1;
        }
    }

    fn ensure_cursor_valid(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        if self.cursor_row >= self.lines.len() {
            self.cursor_row = self.lines.len() - 1;
        }
        self.clamp_cursor_col();
    }

    fn clamp_cursor_col(&mut self) {
        let max_col = char_len(&self.lines[self.cursor_row]);
        if self.cursor_col > max_col {
            self.cursor_col = max_col;
        }
    }
}

/// Count characters (not bytes) in a string.
fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Convert a character column index to a byte offset.
fn char_col_to_byte(s: &str, col: usize) -> usize {
    s.char_indices()
        .nth(col)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
