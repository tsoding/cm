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

    pub fn render(&self, Row { x, y, w }: Row, cursor: &mut Cursor) {
        let begin = self.cursor_x / w * w;
        let end = usize::min(begin + w, self.buffer.len());
        mv(y as i32, x as i32);
        for _ in 0..w {
            addstr(" ");
        }
        mv(y as i32, x as i32);
        addstr(&self.buffer.get(begin..end).unwrap_or(""));
        mv(y as i32, (x + self.cursor_x % w) as i32);

        cursor.x = (x + self.cursor_x) as i32;
        cursor.y = y as i32;
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

    pub fn handle_key(&mut self, key_stroke: KeyStroke, key_map: &KeyMap) {
        // TODO(#187): EditField does not support unicode
        // here is a thing about keystroke.key, i mean the key number 32. can it cause problems here?
        // check comment in line 324 in key_map.rs
        if 32 <= key_stroke.key && key_stroke.key <= 126 {
            self.insert_char(key_stroke.key as u8 as char);
        } else if key_map.is_bound(key_stroke, action::RIGHT) {
            self.right();
        } else if key_map.is_bound(key_stroke, action::LEFT) {
            self.left();
        } else if key_map.is_bound(key_stroke, action::BACK_DELETE) {
            self.backspace();
        }
    }
}
