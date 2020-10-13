use super::*;
use ncurses::*;
use std::collections::BTreeSet;
use std::io;
use std::string::ToString;
use std::mem::{MaybeUninit, transmute};

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
            key_map: {
                let mut key_map: [MaybeUninit<BTreeSet<KeyStroke>>; action::LEN] = unsafe {
                    MaybeUninit::uninit().assume_init()
                };

                for elem in &mut key_map[..] {
                    *elem = MaybeUninit::new(Default::default());
                }

                unsafe { transmute::<_, [BTreeSet<KeyStroke>; action::LEN]>(key_map) }
            },
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
        result.bind(
            KeyStroke {
                key: '/' as i32,
                alt: false,
            },
            action::START_SEARCH,
        );
        result.bind(
            KeyStroke {
                key: 'g' as i32,
                alt: false,
            },
            action::JUMP_TO_START,
        );
        result.bind(
            KeyStroke {
                key: 'G' as i32,
                alt: false,
            },
            action::JUMP_TO_END,
        );
        result.bind(
            KeyStroke {
                key: 'n' as i32,
                alt: false,
            },
            action::NEXT_SEARCH_MATCH,
        );
        result.bind(
            KeyStroke {
                key: 'N' as i32,
                alt: false,
            },
            action::PREV_SEARCH_MATCH,
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
