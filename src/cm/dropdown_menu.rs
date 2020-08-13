use super::*;
use ncurses::*;

pub struct DropdownMenu {
    list: ItemList,
}

// TODO: It would be cool if DropdownMenu actually had an unfolded state
impl DropdownMenu {
    pub fn new() -> Self {
        Self {
            list: ItemList::new(),
        }
    }

    pub fn render(&self, Row{x, y, w}: Row) {
        let s = match self.list.current_item() {
            Some(item) => item,
            None => "<None>"
        };
        mv(y as i32, x as i32);
        addstr(s.get(..w).unwrap_or(s));
    }

    pub fn push(&mut self, item: String) {
        self.list.items.push(item)
    }

    pub fn handle_key(&mut self, key_stroke: &KeyStroke, key_map: &KeyMap) {
        if key_map.is_bound(key_stroke, &Action::Up) {
            self.list.up();
        } else if key_map.is_bound(key_stroke, &Action::Down) {
            self.list.down();
        }
    }
}
