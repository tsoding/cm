use super::*;

pub struct KeyMapSettings {
    pub list: ItemList,
    pub selector: KeyStrokeSelector,
}

impl KeyMapSettings {
    pub fn new() -> Self {
        let mut list = ItemList::new();

        list.items.push(String::from("hello"));
        list.items.push(String::from("world"));
        list.items.push(String::from("foo"));
        list.items.push(String::from("bar"));
        list.items.push(String::from("baz"));

        Self {list, selector: KeyStrokeSelector::new()}
    }

    pub fn render(&self, Rect {x, y, w, h: _}: Rect, _focused: bool) {
        // self.list.render(rect, focused)
        // TODO: introduce some sort of functions to Rect that takes nth row of the Rect
        self.selector.render(Row {x, y, w});
    }

    pub fn handle_key(&mut self, key_stroke: &KeyStroke, key_map: &KeyMap, global: &mut Global) {
        if !global.handle_key(key_stroke, key_map) {
            if key_map.is_bound(key_stroke, &Action::Cancel) {
                global.key_map_settings = false;
            } else {
                // self.list.handle_key(key_stroke, key_map);
                self.selector.handle_key(key_stroke, key_map);
            }
        }
    }
}
