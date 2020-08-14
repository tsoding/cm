use ncurses::*;
use std::collections::BTreeSet;
use std::io;
use std::str::FromStr;
use std::string::ToString;
use super::action;

pub const KEY_ESCAPE: i32 = 0x1B;

// TODO(#152): KeyMap is not configuration right from the application
pub struct KeyMap {
    // NOTE: We are using BTreeSet here for a consistent
    // ordering when we are saving the KeyMap to the configuration
    // file. See KeyMap::to_file().
    key_map: [BTreeSet<KeyStroke>; action::LEN],
}

impl KeyMap {
    pub fn new() -> Self {
        Self {
            key_map: Default::default(),
        }
    }

    pub fn initial() -> Self {
        let mut result = Self::new();
        result.bind(
            KeyStroke {
                key: KEY_UP,
                alt: false,
            },
            action::UP,
        );
        result.bind(
            KeyStroke {
                key: 'k' as i32,
                alt: false,
            },
            action::UP,
        );
        result.bind(
            KeyStroke {
                key: KEY_DOWN,
                alt: false,
            },
            action::DOWN,
        );
        result.bind(
            KeyStroke {
                key: 'j' as i32,
                alt: false,
            },
            action::DOWN,
        );
        result.bind(
            KeyStroke {
                key: KEY_LEFT,
                alt: false,
            },
            action::LEFT,
        );
        result.bind(
            KeyStroke {
                key: 'h' as i32,
                alt: false,
            },
            action::LEFT,
        );
        result.bind(
            KeyStroke {
                key: KEY_RIGHT,
                alt: false,
            },
            action::RIGHT,
        );
        result.bind(
            KeyStroke {
                key: 'l' as i32,
                alt: false,
            },
            action::RIGHT,
        );
        result.bind(
            KeyStroke {
                key: KEY_HOME,
                alt: false,
            },
            action::HOME,
        );
        result.bind(
            KeyStroke {
                key: '0' as i32,
                alt: false,
            },
            action::HOME,
        );
        result.bind(
            KeyStroke {
                key: KEY_DC,
                alt: false,
            },
            action::DELETE,
        );
        result.bind(
            KeyStroke {
                key: 'd' as i32,
                alt: false,
            },
            action::DELETE,
        );
        result.bind(
            KeyStroke {
                key: KEY_BACKSPACE,
                alt: false,
            },
            action::BACK_DELETE,
        );
        result.bind(
            KeyStroke {
                key: 'e' as i32,
                alt: false,
            },
            action::TOGGLE_PROFILE_PANEL,
        );
        result.bind(
            KeyStroke {
                key: 'q' as i32,
                alt: false,
            },
            action::QUIT,
        );
        result.bind(
            KeyStroke {
                key: '\t' as i32,
                alt: false,
            },
            action::FOCUS_FORWARD,
        );
        result.bind(
            KeyStroke {
                key: KEY_BTAB,
                alt: false,
            },
            action::FOCUS_BACKWARD,
        );
        result.bind(
            KeyStroke {
                key: '\n' as i32,
                alt: false,
            },
            action::ACCEPT,
        );
        result.bind(
            KeyStroke {
                key: KEY_ESCAPE,
                alt: false,
            },
            action::CANCEL,
        );
        result.bind(
            KeyStroke {
                key: 'i' as i32,
                alt: true,
            },
            action::DUP_AFTER_ITEM,
        );
        result.bind(
            KeyStroke {
                key: 'I' as i32,
                alt: true,
            },
            action::DUP_BEFORE_ITEM,
        );
        result.bind(
            KeyStroke {
                key: 'i' as i32,
                alt: false,
            },
            action::INSERT_AFTER_ITEM,
        );
        result.bind(
            KeyStroke {
                key: 'I' as i32,
                alt: false,
            },
            action::INSERT_BEFORE_ITEM,
        );
        result.bind(
            KeyStroke {
                key: KEY_F2,
                alt: false,
            },
            action::EDIT_ITEM,
        );
        result.bind(
            KeyStroke {
                key: 'c' as i32,
                alt: false,
            },
            action::EDIT_ITEM,
        );
        result.bind(
            KeyStroke {
                key: '\n' as i32,
                alt: false,
            },
            action::RUN,
        );
        result.bind(
            KeyStroke {
                key: KEY_BACKSPACE,
                alt: false,
            },
            action::BACK,
        );
        result.bind(
            KeyStroke {
                key: KEY_F5,
                alt: false,
            },
            action::RERUN,
        );
        result.bind(
            KeyStroke {
                key: KEY_UP,
                alt: true,
            },
            action::PREV_MATCH,
        );
        result.bind(
            KeyStroke {
                key: 'k' as i32,
                alt: true,
            },
            action::PREV_MATCH,
        );
        result.bind(
            KeyStroke {
                key: KEY_DOWN,
                alt: true,
            },
            action::NEXT_MATCH,
        );
        result.bind(
            KeyStroke {
                key: 'j' as i32,
                alt: true,
            },
            action::NEXT_MATCH,
        );
        result.bind(
            KeyStroke {
                key: KEY_F3,
                alt: false,
            },
            action::EDIT_CMDLINE,
        );
        result.bind(
            KeyStroke {
                key: '!' as i32,
                alt: false,
            },
            action::EDIT_CMDLINE,
        );
        result.bind(
            KeyStroke {
                key: 'K' as i32,
                alt: false,
            },
            action::OPEN_KEY_MAP_SETTINGS,
        );
        result
    }

    pub fn to_file<F: io::Write>(&self, stream: &mut F) -> io::Result<()> {
        for (action_index, action_name) in action::NAMES.iter().enumerate() {
            for key in self.key_map[action_index].iter() {
                writeln!(stream, "{} = {}", key.to_string(), action_name)?;
            }
        }
        Ok(())
    }

    pub fn bind(&mut self, key: KeyStroke, action: action::Type) {
        self.key_map.get_mut(action).map(|x| x.insert(key));
    }

    pub fn is_bound(&self, key: KeyStroke, action: action::Type) -> bool {
        self.key_map.get(action).and_then(|x| x.get(&key)).is_some()
    }

    pub fn keys_of_action(&self, action: action::Type) -> Vec<KeyStroke> {
        let mut result = Vec::new();
        if let Some(keys) = self.key_map.get(action) {
            for key in keys.iter() {
                result.push(*key)
            }
        }
        result
    }

    pub fn update_keys_of_action(&mut self, action: action::Type, new_keys: &[KeyStroke]) {
        if let Some(keys) = self.key_map.get_mut(action) {
            keys.clear();
            for key in new_keys {
                keys.insert(*key);
            }
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
