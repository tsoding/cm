pub const KEY_E: i32 = 0x65;
pub const KEY_Q: i32 = 0x71;
pub const KEY_TAB: i32 = 0x09;
pub const KEY_RETURN: i32 = 0x0a;
pub const KEY_S: i32 = 0x73;
pub const KEY_W: i32 = 0x77;
pub const KEY_D: i32 = 0x64;
pub const KEY_A: i32 = 0x61;
pub const KEY_I: i32 = 0x69;
pub const KEY_ESCAPE: i32 = 0x1B;

#[derive(Debug, Clone, Copy)]
pub struct KeyStroke {
    pub key: i32,
    pub alt: bool,
}

pub struct KeyEscaper {
    pub escape: bool,
}

impl KeyEscaper {
    pub fn new() -> Self {
        Self {
            escape: false
        }
    }

    // REFERENCE: https://en.wikipedia.org/wiki/ANSI_escape_code#Terminal_input_sequences
    pub fn feed(&mut self, key: i32) -> Option<KeyStroke> {
        if self.escape {
            self.escape = false;
            if key == KEY_ESCAPE {
                Some(KeyStroke {
                    key: KEY_ESCAPE,
                    alt: false,
                })
            } else {
                Some(KeyStroke {
                    key: key,
                    alt: true
                })
            }
        } else {
            if key == KEY_ESCAPE {
                self.escape = true;
                None
            } else {
                Some(KeyStroke {
                    key: key,
                    alt: false
                })
            }
        }
    }
}
