use ncurses::*;
use std::collections::{HashMap, HashSet};

pub const KEY_E: i32 = 0x65;
pub const KEY_Q: i32 = 0x71;
pub const KEY_TAB: i32 = 0x09;
pub const KEY_RETURN: i32 = 0x0a;
pub const KEY_S: i32 = 0x73;
pub const KEY_W: i32 = 0x77;
pub const KEY_D: i32 = 0x64;
pub const KEY_A: i32 = 0x61;
pub const KEY_I: i32 = 0x69;
pub const KEY_J: i32 = 0x4A;
pub const KEY_K: i32 = 0x4B;
pub const KEY_SHIFT_I: i32 = 0x49;
pub const KEY_ESCAPE: i32 = 0x1B;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Home,
    InsertAfterItem,
    InsertBeforeItem,
    Delete,
    BackDelete,
    EditItem,
    DupAfterItem,
    DupBeforeItem,
    ToggleProfilePanel,
    Quit,
    FocusForward,
    FocusBackward
}

pub struct KeyMap {
    key_map: HashMap<KeyStroke, HashSet<Action>>,
}

impl KeyMap {
    pub fn new() -> Self {
        Self {
            key_map: HashMap::new()
        }
    }

    pub fn bind(&mut self, key: KeyStroke, action: Action) {
        if let Some(actions) = self.key_map.get_mut(&key) {
            actions.insert(action);
        } else {
            let mut actions = HashSet::new();
            actions.insert(action);
            self.key_map.insert(key, actions);
        }
    }

    pub fn is_bound(&self, key: &KeyStroke, action: &Action) -> bool {
        self.key_map.get(key).and_then(|actions| actions.get(action)).is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyStroke {
    pub key: i32,
    pub alt: bool,
}

impl KeyStroke {
    pub fn get() -> Option<Self> {
        let key = getch();
        if key != -1 {
            if key == KEY_ESCAPE {
                let key1 = getch();
                if key1 != -1 {
                    Some(Self {
                        key: key1,
                        alt: true,
                    })
                } else {
                    Some(Self { key, alt: false })
                }
            } else {
                Some(Self { key, alt: false })
            }
        } else {
            None
        }
    }
}
