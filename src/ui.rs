pub mod keycodes;
pub mod style;

use keycodes::*;
use ncurses::*;
use std::cmp::{max, min};
use style::*;

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
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[derive(Clone, Copy)]
pub struct Row {
    pub x: usize,
    pub y: usize,
    pub w: usize,
}

pub struct EditField {
    pub cursor_x: usize,
    pub buffer: String,
}

impl EditField {
    pub fn new() -> Self {
        Self {
            cursor_x: 0,
            buffer: String::new(),
        }
    }

    pub fn render(&self, Row { x, y, w }: Row) {
        let begin = self.cursor_x / w * w;
        let end = usize::min(begin + w, self.buffer.len());
        mv(y as i32, x as i32);
        for _ in 0..w {
            addstr(" ");
        }
        mv(y as i32, x as i32);
        addstr(&self.buffer.get(begin..end).unwrap_or(""));
        mv(y as i32, (x + self.cursor_x % w) as i32);
    }

    pub fn handle_key(&mut self, key_stroke: KeyStroke) {
        if 32 <= key_stroke.key && key_stroke.key <= 126 {
            self.buffer.insert(self.cursor_x, key_stroke.key as u8 as char);
            self.cursor_x += 1;
        }

        match key_stroke {
            KeyStroke { key: KEY_RIGHT, .. } if self.cursor_x < self.buffer.len() => self.cursor_x += 1,
            KeyStroke { key: KEY_LEFT, .. } if self.cursor_x > 0 => self.cursor_x -= 1,
            KeyStroke { key: KEY_BACKSPACE, .. } if self.cursor_x > 0 => {
                self.cursor_x -= 1;
                self.buffer.remove(self.cursor_x);
            }
            _ => {}
        }
    }
}
