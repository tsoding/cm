use super::*;
use ncurses::*;

#[derive(PartialEq, Clone, Copy)]
pub enum Focus {
    Lines = 0,
    Regexs = 1,
    Cmds = 2,
}

const FOCUS_COUNT: usize = 3;

impl Focus {
    pub fn from_number(n: usize) -> Option<Focus> {
        match n {
            0 => Some(Focus::Lines),
            1 => Some(Focus::Regexs),
            2 => Some(Focus::Cmds),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        Focus::from_number((self as usize + 1) % FOCUS_COUNT).unwrap()
    }

    pub fn prev(self) -> Self {
        let mut result = self as usize;

        if result == 0 {
            result = FOCUS_COUNT;
        }

        Focus::from_number(result - 1).unwrap()
    }
}

pub struct Global {
    /// Indicates that the Profile Panel, that contains Regex and Cmd
    /// lists is visible
    pub profile_pane: bool,
    /// Indicates that the application should quit the main event loop
    /// as soon as possible
    pub quit: bool,
    pub focus: Focus,
}

impl Global {
    pub fn handle_key(&mut self, key_stroke: KeyStroke) -> bool {
        match key_stroke {
            KeyStroke { key: KEY_E, .. } => {
                self.profile_pane = !self.profile_pane;
                true
            }
            KeyStroke { key: KEY_Q, .. } => {
                self.quit = true;
                true
            }
            KeyStroke { key: KEY_TAB, .. } => {
                self.focus = self.focus.next();
                true
            }
            KeyStroke { key: KEY_BTAB, .. } => {
                self.focus = self.focus.prev();
                true
            }
            _ => false,
        }
    }
}
