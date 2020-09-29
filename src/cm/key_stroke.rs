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
        match getch() {
            -1 => None,
            KEY_ESCAPE => match getch() {
                -1 => Some(Self {
                    key: KEY_ESCAPE,
                    alt: false,
                }),
                key => Some(Self { key, alt: true }),
            },
            key => Some(Self { key, alt: false }),
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
                    let key_code = key_of_name(key)?;
                    Ok(KeyStroke {
                        key: key_code,
                        alt: true,
                    })
                }
                [_, unknown] => Err(format!("{} is unknown key modifier", unknown)),
                [key] => {
                    let key_code = key_of_name(key)?;
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

pub const ASCII_KEY_NAMES: [&str; 128] = [
    "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR",
    "SO", "SI", "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB", "ESC",
    "FS", "GS", "RS", "US", "SPACE", "BANG", "DQUOTE", "HASH", "DOLLAR", "PERCENT", "AMPR",
    "QUOTE", "LPRN", "RPRN", "ASTR", "PLUS", "COMMA", "MINUS", "DOT", "SLASH", "0", "1", "2", "3",
    "4", "5", "6", "7", "8", "9", "COLON", "SCOLON", "LT", "EQUAL", "GT", "QUES", "AT", "A", "B",
    "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U",
    "V", "W", "X", "Y", "Z", "LBRACKET", "BSLASH", "RBRACKET", "HAT", "UNDS", "BTICK", "a", "b",
    "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u",
    "v", "w", "x", "y", "z", "LCURLY", "PIPE", "RCURLY", "TILDE", "DEL",
];

pub const NCURSES_KEY_NAMES: [(i32, &str); 110] = [
    (KEY_CODE_YES, "CODE_YES"),
    (KEY_MIN, "MIN"),
    (KEY_BREAK, "BREAK"),
    (KEY_SRESET, "SRESET"),
    (KEY_RESET, "RESET"),
    (KEY_DOWN, "DOWN"),
    (KEY_UP, "UP"),
    (KEY_LEFT, "LEFT"),
    (KEY_RIGHT, "RIGHT"),
    (KEY_HOME, "HOME"),
    (KEY_BACKSPACE, "BACKSPACE"),
    (KEY_F0, "F0"),
    (KEY_F1, "F1"),
    (KEY_F2, "F2"),
    (KEY_F3, "F3"),
    (KEY_F4, "F4"),
    (KEY_F5, "F5"),
    (KEY_F6, "F6"),
    (KEY_F7, "F7"),
    (KEY_F8, "F8"),
    (KEY_F9, "F9"),
    (KEY_F10, "F10"),
    (KEY_F11, "F11"),
    (KEY_F12, "F12"),
    (KEY_F13, "F13"),
    (KEY_F14, "F14"),
    (KEY_F15, "F15"),
    (KEY_DL, "DL"),
    (KEY_IL, "IL"),
    (KEY_DC, "DC"),
    (KEY_IC, "IC"),
    (KEY_EIC, "EIC"),
    (KEY_CLEAR, "CLEAR"),
    (KEY_EOS, "EOS"),
    (KEY_EOL, "EOL"),
    (KEY_SF, "SF"),
    (KEY_SR, "SR"),
    (KEY_NPAGE, "NPAGE"),
    (KEY_PPAGE, "PPAGE"),
    (KEY_STAB, "STAB"),
    (KEY_CTAB, "CTAB"),
    (KEY_CATAB, "CATAB"),
    (KEY_ENTER, "ENTER"),
    (KEY_PRINT, "PRINT"),
    (KEY_LL, "LL"),
    (KEY_A1, "A1"),
    (KEY_A3, "A3"),
    (KEY_B2, "B2"),
    (KEY_C1, "C1"),
    (KEY_C3, "C3"),
    (KEY_BTAB, "BTAB"),
    (KEY_BEG, "BEG"),
    (KEY_CANCEL, "CANCEL"),
    (KEY_CLOSE, "CLOSE"),
    (KEY_COMMAND, "COMMAND"),
    (KEY_COPY, "COPY"),
    (KEY_CREATE, "CREATE"),
    (KEY_END, "END"),
    (KEY_EXIT, "EXIT"),
    (KEY_FIND, "FIND"),
    (KEY_HELP, "HELP"),
    (KEY_MARK, "MARK"),
    (KEY_MESSAGE, "MESSAGE"),
    (KEY_MOVE, "MOVE"),
    (KEY_NEXT, "NEXT"),
    (KEY_OPEN, "OPEN"),
    (KEY_OPTIONS, "OPTIONS"),
    (KEY_PREVIOUS, "PREVIOUS"),
    (KEY_REDO, "REDO"),
    (KEY_REFERENCE, "REFERENCE"),
    (KEY_REFRESH, "REFRESH"),
    (KEY_REPLACE, "REPLACE"),
    (KEY_RESTART, "RESTART"),
    (KEY_RESUME, "RESUME"),
    (KEY_SAVE, "SAVE"),
    (KEY_SBEG, "SBEG"),
    (KEY_SCANCEL, "SCANCEL"),
    (KEY_SCOMMAND, "SCOMMAND"),
    (KEY_SCOPY, "SCOPY"),
    (KEY_SCREATE, "SCREATE"),
    (KEY_SDC, "SDC"),
    (KEY_SDL, "SDL"),
    (KEY_SELECT, "SELECT"),
    (KEY_SEND, "SEND"),
    (KEY_SEOL, "SEOL"),
    (KEY_SEXIT, "SEXIT"),
    (KEY_SFIND, "SFIND"),
    (KEY_SHELP, "SHELP"),
    (KEY_SHOME, "SHOME"),
    (KEY_SIC, "SIC"),
    (KEY_SLEFT, "SLEFT"),
    (KEY_SMESSAGE, "SMESSAGE"),
    (KEY_SMOVE, "SMOVE"),
    (KEY_SNEXT, "SNEXT"),
    (KEY_SOPTIONS, "SOPTIONS"),
    (KEY_SPREVIOUS, "SPREVIOUS"),
    (KEY_SPRINT, "SPRINT"),
    (KEY_SREDO, "SREDO"),
    (KEY_SREPLACE, "SREPLACE"),
    (KEY_SRIGHT, "SRIGHT"),
    (KEY_SRSUME, "SRSUME"),
    (KEY_SSAVE, "SSAVE"),
    (KEY_SSUSPEND, "SSUSPEND"),
    (KEY_SUNDO, "SUNDO"),
    (KEY_SUSPEND, "SUSPEND"),
    (KEY_UNDO, "UNDO"),
    (KEY_MOUSE, "MOUSE"),
    (KEY_RESIZE, "RESIZE"),
    (KEY_EVENT, "EVENT"),
    (KEY_MAX, "MAX"),
];

fn name_of_key(key: i32) -> String {
    if (key as usize) < ASCII_KEY_NAMES.len() {
        String::from(ASCII_KEY_NAMES[key as usize])
    } else if let Some((_, name)) = NCURSES_KEY_NAMES.iter().find(|(code, _)| *code == key) {
        String::from(*name)
    } else {
        format!("#{}", key)
    }
}

pub fn key_of_name(name: &str) -> Result<i32, String> {
    if let Some((key, _)) = ASCII_KEY_NAMES
        .iter()
        .enumerate()
        .find(|(_, &ascii_name)| ascii_name == name)
    {
        Ok(key as i32)
    } else if let Some((key, _)) = NCURSES_KEY_NAMES
        .iter()
        .find(|(_, ncurses_name)| *ncurses_name == name)
    {
        Ok(*key as i32)
    } else if name.starts_with('#') {
        name[1..].parse::<i32>().map_err(|e| e.to_string())
    } else {
        Err("Not a key name".to_string())
    }
}

impl ToString for KeyStroke {
    fn to_string(&self) -> String {
        // TODO(#156): Human readable KeyStroke serialization format is required
        format!(
            "key:{}{}",
            name_of_key(self.key),
            if self.alt { ",alt" } else { "" }
        )
    }
}
