use super::*;
use ncurses::*;
use pcre2::bytes::Regex;
use std::cmp::{max, min};

pub struct ItemList {
    pub items: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl ItemList {
    pub fn new() -> Self {
        Self {
            items: Vec::<String>::new(),
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1
        }
    }

    pub fn down(&mut self) {
        if self.cursor_y + 1 < self.items.len() {
            self.cursor_y += 1;
        }
    }

    pub fn left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    pub fn right(&mut self) {
        self.cursor_x += 1;
    }

    pub fn home(&mut self) {
        self.cursor_x = 0;
    }

    pub fn delete_current(&mut self) {
        if self.cursor_y < self.items.len() {
            self.items.remove(self.cursor_y);
            if !self.items.is_empty() {
                self.cursor_y = min(max(0, self.cursor_y), self.items.len() - 1);
            }
        }
    }

    pub fn insert_before_current(&mut self, line: String) {
        self.items.insert(self.cursor_y, line);
    }

    pub fn insert_after_current(&mut self, line: String) {
        if !self.items.is_empty() {
            self.cursor_y += 1;
        }

        self.items.insert(self.cursor_y, line);
    }

    pub fn duplicate_after(&mut self) {
        if let Some(item) = self.current_item().map(String::from) {
            self.insert_after_current(item);
        }
    }

    pub fn duplicate_before(&mut self) {
        if let Some(item) = self.current_item().map(String::from) {
            self.insert_before_current(item);
        }
    }

    pub fn handle_key(&mut self, key_stroke: KeyStroke) {
        match key_stroke {
            KeyStroke { key: KEY_S, .. } => self.down(),
            KeyStroke { key: KEY_W, .. } => self.up(),
            KeyStroke { key: KEY_D, .. } => self.right(),
            KeyStroke { key: KEY_A, .. } => self.left(),
            KeyStroke { key: KEY_DC, .. } => self.delete_current(),
            KeyStroke { key: KEY_HOME, .. } => self.home(),
            _ => {}
        }
    }

    pub fn render(&self, Rect { x, y, w, h }: Rect, focused: bool) {
        if h > 0 {
            // TODO(#16): word wrapping for long lines
            for (i, item) in self
                .items
                .iter()
                .skip(self.cursor_y / h * h)
                .enumerate()
                .take_while(|(i, _)| *i < h)
            {
                let line_to_render = {
                    let mut line_to_render = item
                        .trim_end()
                        .get(self.cursor_x..)
                        .unwrap_or("")
                        .to_string();
                    let n = line_to_render.len();
                    if n < w {
                        for _ in 0..(w - n) {
                            line_to_render.push(' ');
                        }
                    }
                    line_to_render
                };

                mv((y + i) as i32, x as i32);
                let selected = i == (self.cursor_y % h);
                let pair = if selected {
                    if focused {
                        CURSOR_PAIR
                    } else {
                        UNFOCUSED_CURSOR_PAIR
                    }
                } else {
                    REGULAR_PAIR
                };
                attron(COLOR_PAIR(pair));
                addstr(&line_to_render);
                attroff(COLOR_PAIR(pair));
            }
        }
    }

    pub fn current_row(&self, Rect { x, y, w, h }: Rect) -> Row {
        Row {
            x,
            y: self.cursor_y % h + y,
            w,
        }
    }

    pub fn current_item(&self) -> Option<&str> {
        if self.cursor_y < self.items.len() {
            Some(&self.items[self.cursor_y])
        } else {
            None
        }
    }

    pub fn is_at_begin(&self) -> bool {
        self.cursor_y == 0
    }

    pub fn is_at_end(&self) -> bool {
        self.cursor_y >= self.items.len() - 1
    }

    pub fn is_current_line_matches(&mut self, regex: &Regex) -> bool {
        if let Some(item) = self.current_item() {
            regex.is_match(item.as_bytes()).unwrap()
        } else {
            false
        }
    }
}
