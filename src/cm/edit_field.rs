use super::*;
use ncurses::*;

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

    pub fn insert_char(&mut self, ch: char) {
        self.buffer.insert(self.cursor_x, ch);
        self.cursor_x += 1;
    }

    pub fn left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1
        }
    }

    pub fn right(&mut self) {
        if self.cursor_x < self.buffer.len() {
            self.cursor_x += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.buffer.remove(self.cursor_x);
        }
    }

    pub fn handle_key(&mut self, key_stroke: KeyStroke) {
        if 32 <= key_stroke.key && key_stroke.key <= 126 {
            self.insert_char(key_stroke.key as u8 as char);
        }

        match key_stroke {
            KeyStroke { key: KEY_RIGHT, .. } => {
                self.right();
            }
            KeyStroke { key: KEY_LEFT, .. } =>  {
                self.left();
            },
            KeyStroke {
                key: KEY_BACKSPACE, ..
            } => {
                self.backspace();
            }
            _ => {}
        }
    }
}
