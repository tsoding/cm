use ncurses::*;
use std::collections::{HashMap, HashSet};

pub const KEY_ESCAPE: i32 = 0x1B;

// TODO(#145): Separate Delete Character and Delete Item actions
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
    EditCmdline,
}

pub struct KeyMap {
    key_map: HashMap<KeyStroke, HashSet<Action>>,
}

impl KeyMap {
    pub fn new() -> Self {
        Self {
            key_map: HashMap::new(),
        }
    }

    pub fn initial() -> Self {
        let mut result = Self::new();
        result.bind(
            KeyStroke {
                key: KEY_UP,
                alt: false,
            },
            Action::Up,
        );
        result.bind(
            KeyStroke {
                key: 'k' as i32,
                alt: false,
            },
            Action::Up,
        );
        result.bind(
            KeyStroke {
                key: KEY_DOWN,
                alt: false,
            },
            Action::Down,
        );
        result.bind(
            KeyStroke {
                key: 'j' as i32,
                alt: false,
            },
            Action::Down,
        );
        result.bind(
            KeyStroke {
                key: KEY_LEFT,
                alt: false,
            },
            Action::Left,
        );
        result.bind(
            KeyStroke {
                key: 'h' as i32,
                alt: false,
            },
            Action::Left,
        );
        result.bind(
            KeyStroke {
                key: KEY_RIGHT,
                alt: false,
            },
            Action::Right,
        );
        result.bind(
            KeyStroke {
                key: 'l' as i32,
                alt: false,
            },
            Action::Right,
        );
        result.bind(
            KeyStroke {
                key: KEY_HOME,
                alt: false,
            },
            Action::Home,
        );
        result.bind(
            KeyStroke {
                key: '0' as i32,
                alt: false,
            },
            Action::Home,
        );
        result.bind(
            KeyStroke {
                key: KEY_DC,
                alt: false,
            },
            Action::Delete,
        );
        result.bind(
            KeyStroke {
                key: 'd' as i32,
                alt: false,
            },
            Action::Delete,
        );
        result.bind(
            KeyStroke {
                key: KEY_BACKSPACE,
                alt: false,
            },
            Action::BackDelete,
        );
        result.bind(
            KeyStroke {
                key: 'e' as i32,
                alt: false,
            },
            Action::ToggleProfilePanel,
        );
        result.bind(
            KeyStroke {
                key: 'q' as i32,
                alt: false,
            },
            Action::Quit,
        );
        result.bind(
            KeyStroke {
                key: '\t' as i32,
                alt: false,
            },
            Action::FocusForward,
        );
        result.bind(
            KeyStroke {
                key: KEY_BTAB,
                alt: false,
            },
            Action::FocusBackward,
        );
        result.bind(
            KeyStroke {
                key: '\n' as i32,
                alt: false,
            },
            Action::Accept,
        );
        result.bind(
            KeyStroke {
                key: KEY_ESCAPE,
                alt: false,
            },
            Action::Cancel,
        );
        result.bind(
            KeyStroke {
                key: 'i' as i32,
                alt: true,
            },
            Action::DupAfterItem,
        );
        result.bind(
            KeyStroke {
                key: 'I' as i32,
                alt: true,
            },
            Action::DupBeforeItem,
        );
        result.bind(
            KeyStroke {
                key: 'i' as i32,
                alt: false,
            },
            Action::InsertAfterItem,
        );
        result.bind(
            KeyStroke {
                key: 'I' as i32,
                alt: false,
            },
            Action::InsertBeforeItem,
        );
        result.bind(
            KeyStroke {
                key: KEY_F2,
                alt: false,
            },
            Action::EditItem,
        );
        result.bind(
            KeyStroke {
                key: 'c' as i32,
                alt: false,
            },
            Action::EditItem,
        );
        result.bind(
            KeyStroke {
                key: '\n' as i32,
                alt: false,
            },
            Action::Run,
        );
        result.bind(
            KeyStroke {
                key: KEY_BACKSPACE,
                alt: false,
            },
            Action::Back,
        );
        result.bind(
            KeyStroke {
                key: KEY_F5,
                alt: false,
            },
            Action::Rerun,
        );
        result.bind(
            KeyStroke {
                key: KEY_UP,
                alt: true,
            },
            Action::PrevMatch,
        );
        result.bind(
            KeyStroke {
                key: 'k' as i32,
                alt: true,
            },
            Action::PrevMatch,
        );
        result.bind(
            KeyStroke {
                key: KEY_DOWN,
                alt: true,
            },
            Action::NextMatch,
        );
        result.bind(
            KeyStroke {
                key: 'j' as i32,
                alt: true,
            },
            Action::NextMatch,
        );
        result.bind(
            KeyStroke {
                key: KEY_F3,
                alt: false,
            },
            Action::EditCmdline,
        );
        result.bind(
            KeyStroke {
                key: '!' as i32,
                alt: false,
            },
            Action::EditCmdline,
        );
        result
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
        self.key_map
            .get(key)
            .and_then(|actions| actions.get(action))
            .is_some()
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
