use ncurses::*;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::string::ToString;

pub const KEY_ESCAPE: i32 = 0x1B;

// TODO(#145): Separate Delete Character and Delete Item actions
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl FromStr for Action {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "home" => Ok(Self::Home),
            "insert_after_item" => Ok(Self::InsertAfterItem),
            "insert_before_item" => Ok(Self::InsertBeforeItem),
            "delete" => Ok(Self::Delete),
            "back_delete" => Ok(Self::BackDelete),
            "edit_item" => Ok(Self::EditItem),
            "dup_after_item" => Ok(Self::DupAfterItem),
            "dup_before_item" => Ok(Self::DupBeforeItem),
            "toggle_profile_panel" => Ok(Self::ToggleProfilePanel),
            "quit" => Ok(Self::Quit),
            "focus_forward" => Ok(Self::FocusForward),
            "focus_backward" => Ok(Self::FocusBackward),
            "accept" => Ok(Self::Accept),
            "cancel" => Ok(Self::Cancel),
            "run" => Ok(Self::Run),
            "run_into_itself" => Ok(Self::RunIntoItself),
            "rerun" => Ok(Self::Rerun),
            "back" => Ok(Self::Back),
            "next_match" => Ok(Self::NextMatch),
            "prev_match" => Ok(Self::PrevMatch),
            "edit_cmdline" => Ok(Self::EditCmdline),
            unknown => Err(format!("Unknown action `{}`", unknown)),
        }
    }
}

impl ToString for Action {
    fn to_string(&self) -> String {
        let result = match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
            Self::Home => "home",
            Self::InsertAfterItem => "insert_after_item",
            Self::InsertBeforeItem => "insert_before_item",
            Self::Delete => "delete",
            Self::BackDelete => "back_delete",
            Self::EditItem => "edit_item",
            Self::DupAfterItem => "dup_after_item",
            Self::DupBeforeItem => "dup_before_item",
            Self::ToggleProfilePanel => "toggle_profile_panel",
            Self::Quit => "quit",
            Self::FocusForward => "focus_forward",
            Self::FocusBackward => "focus_backward",
            Self::Accept => "accept",
            Self::Cancel => "cancel",
            Self::Run => "run",
            Self::RunIntoItself => "run_into_itself",
            Self::Rerun => "rerun",
            Self::Back => "back",
            Self::NextMatch => "next_match",
            Self::PrevMatch => "prev_match",
            Self::EditCmdline => "edit_cmdline",
        };
        String::from(result)
    }
}

// TODO: KeyMap is not configuration right from the application
pub struct KeyMap {
    // NOTE: We are using BTree{Map, Set} here for a consistent
    // ordering when we are saving the KeyMap to the configuration
    // file. See Profile::to_file().
    pub key_map: BTreeMap<KeyStroke, BTreeSet<Action>>,
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
        result
    }

    pub fn bind(&mut self, key: KeyStroke, action: Action) {
        if let Some(actions) = self.key_map.get_mut(&key) {
            actions.insert(action);
        } else {
            let mut actions = BTreeSet::new();
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        format!("key:{}{}", self.key, if self.alt { ",alt" } else { "" })
    }
}
