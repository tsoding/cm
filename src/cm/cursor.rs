use ncurses::CURSOR_VISIBILITY::{CURSOR_INVISIBLE, CURSOR_VISIBLE};
use ncurses::{curs_set, mv};

pub struct Cursor {
    pub visible: bool,
    pub x: i32,
    pub y: i32,
}

impl Cursor {
    pub fn sync(&self) {
        curs_set(if self.visible {
            CURSOR_VISIBLE
        } else {
            CURSOR_INVISIBLE
        });
        mv(self.y, self.x);
    }
}
