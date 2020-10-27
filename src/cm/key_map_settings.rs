use super::*;
use ncurses::*;

enum State {
    ListOfActions,
    KeysOfAction,
    SelectingKey,
}

pub struct KeyMapSettings {
    state: State,
    pub list_of_actions: ItemList<String>,
    pub keys_of_action: ItemList<KeyStroke>,
}

impl KeyMapSettings {
    pub fn new() -> Self {
        let mut result = Self {
            state: State::ListOfActions,
            list_of_actions: ItemList::new(),
            keys_of_action: ItemList::new(),
        };

        for name in action::NAMES.iter() {
            result.list_of_actions.items.push(String::from(*name));
        }

        result
    }

    pub fn render(&mut self, rect: Rect, focused: bool) {
        match self.state {
            State::ListOfActions => self.list_of_actions.render(rect, focused),
            State::KeysOfAction => {
                let (left, middle) = rect.vertical_split(3);
                self.list_of_actions.render(left, false);
                self.keys_of_action.render(middle, focused);
            }
            State::SelectingKey => {
                let (left, middle) = rect.vertical_split(3);
                self.list_of_actions.render(left, false);
                self.keys_of_action.render(middle, focused);

                let input = "<Input...>";
                let Row { x, y, w } = self.keys_of_action.current_row(middle);
                let pair = if focused {
                    CURSOR_PAIR
                } else {
                    UNFOCUSED_CURSOR_PAIR
                };
                mv(y as i32, x as i32);
                attron(COLOR_PAIR(pair));
                addstr(input.get(..w).unwrap_or(input));
                attroff(COLOR_PAIR(pair));
            }
        }
    }

    pub fn handle_key(&mut self, key_stroke: KeyStroke, key_map: &mut KeyMap, global: &mut Global) {
        if !global.handle_key(key_stroke, key_map) {
            match self.state {
                State::ListOfActions => {
                    if key_map.is_bound(key_stroke, action::BACK) {
                        global.key_map_settings = false;
                    } else if key_map.is_bound(key_stroke, action::UP) {
                        self.list_of_actions.up();
                    } else if key_map.is_bound(key_stroke, action::DOWN) {
                        self.list_of_actions.down();
                    } else if key_map.is_bound(key_stroke, action::ACCEPT) {
                        self.keys_of_action.items.clear();
                        self.keys_of_action.cursor_y = 0;
                        for key_stroke in
                            key_map.keys_of_action(self.list_of_actions.cursor_y).iter()
                        {
                            self.keys_of_action.items.push(*key_stroke);
                        }
                        self.state = State::KeysOfAction;
                    } else {
                        self.list_of_actions.handle_key(key_stroke, key_map);
                    }
                }
                State::KeysOfAction => {
                    if key_map.is_bound(key_stroke, action::BACK) {
                        self.state = State::ListOfActions;
                        key_map.update_keys_of_action(
                            self.list_of_actions.cursor_y,
                            &self.keys_of_action.items,
                        );
                    } else if key_map.is_bound(key_stroke, action::UP) {
                        self.keys_of_action.up();
                    } else if key_map.is_bound(key_stroke, action::DOWN) {
                        self.keys_of_action.down();
                    } else if key_map.is_bound(key_stroke, action::DELETE) {
                        self.keys_of_action.delete_current();
                    } else if key_map.is_bound(key_stroke, action::INSERT_AFTER_ITEM) {
                        self.keys_of_action
                            .insert_after_current(KeyStroke { key: 0, alt: false });
                        self.state = State::SelectingKey;
                    } else if key_map.is_bound(key_stroke, action::INSERT_BEFORE_ITEM) {
                        self.keys_of_action
                            .insert_before_current(KeyStroke { key: 0, alt: false });
                        self.state = State::SelectingKey;
                    } else if key_map.is_bound(key_stroke, action::CANCEL) {
                        self.state = State::ListOfActions;
                    } else {
                        self.keys_of_action.handle_key(key_stroke, key_map);
                    }
                }
                State::SelectingKey => {
                    self.keys_of_action.set_current_item(key_stroke);
                    self.state = State::KeysOfAction;
                }
            }
        }
    }
}
