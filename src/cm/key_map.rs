use ncurses::*;
use std::collections::{HashMap, HashSet};

pub const KEY_ESCAPE: i32 = 0x1B;

// TODO: Separate Delete Character and Delete Item actions
#[derive(Debug, PartialEq, Eq, Hash)]
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
    FocusBackward,
    Accept,
    Cancel,
    Run,
    RunIntoItself,
    Rerun,
    Back,
    NextMatch,
    PrevMatch,
    EditCmdline
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

#[derive(Debug, PartialEq, Eq, Hash)]
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
