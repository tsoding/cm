use super::*;

pub struct KeyMapSettings {
    pub list_of_actions: ItemList,
    pub keys_of_action: ItemList,
    pub editing_action: bool,
}

impl KeyMapSettings {
    pub fn new() -> Self {
        let mut result = Self {
            list_of_actions: ItemList::new(),
            keys_of_action: ItemList::new(),
            editing_action: false,
        };

        for (name, _) in ACTION_NAMES.iter() {
            result.list_of_actions.items.push(String::from(*name));
        }

        result
    }

    pub fn render(&self, rect: Rect, focused: bool) {
        // TODO: introduce some sort of functions to Rect that takes nth row of the Rect
        if self.editing_action {
            self.keys_of_action.render(rect, focused);
        } else {
            self.list_of_actions.render(rect, focused)
        }
    }

    pub fn handle_key(&mut self, key_stroke: &KeyStroke, key_map: &KeyMap, global: &mut Global) {
        if !global.handle_key(key_stroke, key_map) {
            if self.editing_action {
                if key_map.is_bound(key_stroke, &Action::Back) {
                    self.editing_action = false;
                } else if key_map.is_bound(key_stroke, &Action::Up) {
                    self.keys_of_action.up();
                } else if key_map.is_bound(key_stroke, &Action::Down) {
                    self.keys_of_action.down();
                }
            } else {
                if key_map.is_bound(key_stroke, &Action::Back) {
                    global.key_map_settings = false;
                } else if key_map.is_bound(key_stroke, &Action::Up) {
                    self.list_of_actions.up();
                } else if key_map.is_bound(key_stroke, &Action::Down) {
                    self.list_of_actions.down();
                } else if key_map.is_bound(key_stroke, &Action::Accept) {
                    self.keys_of_action.items.clear();
                    for key_stroke in key_map.keys_of_action(&ACTION_NAMES[self.list_of_actions.cursor_y].1).iter() {
                        self.keys_of_action.items.push(key_stroke.to_string());
                    }
                    self.editing_action = true;
                }
            }
        }
    }
}
