use ncurses::*;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::string::ToString;

pub const KEY_ESCAPE: i32 = 0x1B;

// TODO(#145): Separate Delete Character and Delete Item actions
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
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
    OpenKeyMapSettings,
}

pub const ACTION_NAMES: [(&str, Action); 26] = [
    ("up", Action::Up),
    ("down", Action::Down),
    ("left", Action::Left),
    ("right", Action::Right),
    ("home", Action::Home),
    ("insert_after_item", Action::InsertAfterItem),
    ("insert_before_item", Action::InsertBeforeItem),
    ("delete", Action::Delete),
    ("back_delete", Action::BackDelete),
    ("edit_item", Action::EditItem),
    ("dup_after_item", Action::DupAfterItem),
    ("dup_before_item", Action::DupBeforeItem),
    ("toggle_profile_panel", Action::ToggleProfilePanel),
    ("quit", Action::Quit),
    ("focus_forward", Action::FocusForward),
    ("focus_backward", Action::FocusBackward),
    ("accept", Action::Accept),
    ("cancel", Action::Cancel),
    ("run", Action::Run),
    ("run_into_itself", Action::RunIntoItself),
    ("rerun", Action::Rerun),
    ("back", Action::Back),
    ("next_match", Action::NextMatch),
    ("prev_match", Action::PrevMatch),
    ("edit_cmdline", Action::EditCmdline),
    ("open_key_map_settings", Action::OpenKeyMapSettings),
];

impl FromStr for Action {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ACTION_NAMES
            .iter()
            .find(|(name, _)| *name == s)
            .map(|(_, value)| *value)
            .ok_or(format!("Unknown action `{}`", s))
    }
}

impl ToString for Action {
    fn to_string(&self) -> String {
        ACTION_NAMES
            .iter()
            .find(|(_, value)| value == self)
            .map(|(name, _)| String::from(*name))
            .unwrap()
    }
}

// TODO(#152): KeyMap is not configuration right from the application
pub struct KeyMap {
    // NOTE: We are using BTree{Map, Set} here for a consistent
    // ordering when we are saving the KeyMap to the configuration
    // file. See Profile::to_file().
    pub key_map: BTreeMap<Action, BTreeSet<KeyStroke>>,
}

impl KeyMap {
    pub fn new() -> Self {
        Self {
            key_map: BTreeMap::new(),
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
        result.bind(
            KeyStroke {
                key: 'K' as i32,
                alt: false,
            },
            Action::OpenKeyMapSettings,
        );
        result
    }

    pub fn bind(&mut self, key: KeyStroke, action: Action) {
        if let Some(keys) = self.key_map.get_mut(&action) {
            keys.insert(key);
        } else {
            let mut keys = BTreeSet::new();
            keys.insert(key);
            self.key_map.insert(action, keys);
        }
    }

    pub fn is_bound(&self, key: KeyStroke, action: Action) -> bool {
        self.key_map
            .get(&action)
            .and_then(|keys| keys.get(&key))
            .is_some()
    }

    pub fn keys_of_action(&self, action: Action) -> Vec<KeyStroke> {
        let mut result = Vec::new();
        if let Some(keys) = self.key_map.get(&action) {
            for key in keys.iter() {
                result.push(*key)
            }
        }
        result
    }

    pub fn update_keys_of_action(&mut self, action: Action, new_keys: &[KeyStroke]) {
        let keys = self.key_map.entry(action).or_insert_with(BTreeSet::new);
        keys.clear();
        for key in new_keys {
            keys.insert(*key);
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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

fn split(s: &str, delim: char) -> Vec<&str> {
    s.split(delim).map(|s| s.trim()).collect::<Vec<&str>>()
}

impl FromStr for KeyStroke {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match split(s, ':').as_slice() {
            ["key", params] => match split(params, ',').as_slice() {
                [key, "alt"] => {
                    let key_code = key.parse::<i32>().map_err(|e| e.to_string())?;
                    Ok(KeyStroke {
                        key: key_code,
                        alt: true,
                    })
                }
                [_, unknown] => Err(format!("{} is unknown key modifier", unknown)),
                [key] => {
                    let key_code = key.parse::<i32>().map_err(|e| e.to_string())?;
                    Ok(KeyStroke {
                        key: key_code,
                        alt: false,
                    })
                }
                _ => Err(String::from("Could not parse key stroke")),
            },
            [unknown, ..] => Err(format!("Unknown key prefix `{}`", unknown)),
            _ => Err("Could not parse key".to_string()),
        }
    }
}

impl ToString for KeyStroke {
    fn to_string(&self) -> String {
        // TODO(#156): Human readable KeyStroke serialization format is required
        format!("key:{}{}", self.key, if self.alt { ",alt" } else { "" })
    }
}
