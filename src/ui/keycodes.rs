use ncurses::*;

pub const KEY_E: i32 = 0x65;
pub const KEY_Q: i32 = 0x71;
pub const KEY_TAB: i32 = 0x09;
pub const KEY_RETURN: i32 = 0x0a;
pub const KEY_S: i32 = 0x73;
pub const KEY_W: i32 = 0x77;
pub const KEY_D: i32 = 0x64;
pub const KEY_A: i32 = 0x61;
pub const KEY_I: i32 = 0x69;
pub const KEY_SHIFT_I: i32 = 0x49;
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
        Self { escape: false }
    }

    // REFERENCE: https://en.wikipedia.org/wiki/ANSI_escape_code#Terminal_input_sequences
    pub fn feed(&mut self) -> Option<KeyStroke> {
        let key = getch();
        if key != -1 {
            if key == KEY_ESCAPE {
                let key1 = getch();
                if key1 != -1 {
                    Some(KeyStroke{key:key1, alt: true})
                } else {
                    Some(KeyStroke{key, alt: false})
                }
            } else {
                Some(KeyStroke{key, alt: false})
            }
        } else {
            None
        }
    }
}
