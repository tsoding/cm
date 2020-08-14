use ncurses::*;
use std::str::FromStr;

pub const KEY_ESCAPE: i32 = 0x1B;

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
