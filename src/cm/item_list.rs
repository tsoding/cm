use super::*;
use ncurses::*;
use pcre2::bytes::Regex;
use std::cmp::{max, min};

pub struct ItemList<T: ToString + Clone> {
    pub items: Vec<T>,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub cursor_y: usize,
}

impl<T: ToString + Clone> ItemList<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            scroll_x: 0,
            scroll_y: 0,
            cursor_y: 0,
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        if page_size > self.scroll_y {
            self.scroll_y = 0;
        } else {
            self.scroll_y -= page_size;
        }
        self.cursor_y = self.scroll_y;
    }

    pub fn page_down(&mut self, page_size: usize) {
        self.scroll_y = min(self.scroll_y + page_size, self.items.len() - 1);
        self.cursor_y = min(self.scroll_y + page_size - 1, self.items.len() - 1);
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
        if self.scroll_x > 0 {
            self.scroll_x -= 1;
        }
    }

    pub fn right(&mut self) {
        self.scroll_x += 1;
    }

    pub fn home(&mut self) {
        self.scroll_x = 0;
    }

    pub fn delete_current(&mut self) {
        if self.cursor_y < self.items.len() {
            self.items.remove(self.cursor_y);
            if !self.items.is_empty() {
                self.cursor_y = min(max(0, self.cursor_y), self.items.len() - 1);
            }
        }
    }

    pub fn insert_before_current(&mut self, line: T) {
        self.items.insert(self.cursor_y, line);
    }

    pub fn insert_after_current(&mut self, line: T) {
        if !self.items.is_empty() {
            self.cursor_y += 1;
        }

        self.items.insert(self.cursor_y, line);
    }

    pub fn duplicate_after(&mut self) {
        if let Some(item) = self.current_item().cloned() {
            self.insert_after_current(item);
        }
    }

    pub fn duplicate_before(&mut self) {
        if let Some(item) = self.current_item().cloned() {
            self.insert_before_current(item);
        }
    }

    pub fn jump_to_start(&mut self) {
        self.cursor_y = 0;
    }

    pub fn jump_to_end(&mut self) {
        self.cursor_y = self.items.len() - 1;
    }

    pub fn handle_key(&mut self, key_stroke: KeyStroke, key_map: &KeyMap) {
        // TODO: page-up/page-down page size is hardcoded
        const PAGE_SIZE: usize = 30;

        if key_map.is_bound(key_stroke, action::DOWN) {
            self.down();
        } else if key_map.is_bound(key_stroke, action::UP) {
            self.up();
        } else if key_map.is_bound(key_stroke, action::RIGHT) {
            self.right();
        } else if key_map.is_bound(key_stroke, action::LEFT) {
            self.left();
        } else if key_map.is_bound(key_stroke, action::HOME) {
            self.home();
        } else if key_map.is_bound(key_stroke, action::JUMP_TO_START) {
            self.jump_to_start();
        } else if key_map.is_bound(key_stroke, action::JUMP_TO_END) {
            self.jump_to_end();
        } else if key_map.is_bound(key_stroke, action::PAGE_UP) {
            self.page_up(PAGE_SIZE);
        } else if key_map.is_bound(key_stroke, action::PAGE_DOWN) {
            self.page_down(PAGE_SIZE);
        }
    }

    pub fn sync_scroll_y(&mut self, h: usize) {
        if self.cursor_y >= self.scroll_y + h {
            self.scroll_y = self.cursor_y - h + 1;
        } else if self.cursor_y < self.scroll_y {
            self.scroll_y = self.cursor_y;
        }
    }

    pub fn render(&mut self, Rect { x, y, w, h }: Rect, focused: bool) {
        if h > 0 {
            self.sync_scroll_y(h);
            // TODO(#16): word wrapping for long lines
            for i in 0..h {
                if self.scroll_y + i < self.items.len() {
                    let s = self.items[self.scroll_y + i].to_string();
                    let (line_to_render, (left, right)) =
                        unicode::width_substr(s.trim_end(), self.scroll_x..self.scroll_x + w)
                            .unwrap();

                    mv((y + i) as i32, x as i32);
                    let selected = self.scroll_y + i == self.cursor_y;
                    // TODO(#188): item list selection does not extend until the end of the screen
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
                    for _ in 0..left {
                        addstr(" ");
                    }
                    // addstr(&format!("{:?}", (left, right)));
                    addstr(&line_to_render);
                    for _ in 0..right {
                        addstr(" ");
                    }
                    attroff(COLOR_PAIR(pair));
                }
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

    pub fn set_current_item(&mut self, item: T) {
        if self.cursor_y < self.items.len() {
            self.items[self.cursor_y] = item;
        }
    }

    pub fn current_item(&self) -> Option<&T> {
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
            regex.is_match(item.to_string().as_bytes()).unwrap()
        } else {
            false
        }
    }
}
