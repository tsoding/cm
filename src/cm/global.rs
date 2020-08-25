use super::*;

#[derive(PartialEq, Clone, Copy)]
pub enum Focus {
    Output = 0,
    Regexs = 1,
    Cmds = 2,
}

const FOCUS_COUNT: usize = 3;

impl Focus {
    pub fn from_number(n: usize) -> Option<Focus> {
        match n {
            0 => Some(Focus::Output),
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

#[derive(PartialEq)]
pub enum BottomState {
    Nothing,
    Cmdline,
    Search,
}

pub struct Global {
    /// Indicates that the Profile Panel, that contains Regex and Cmd
    /// lists is visible
    pub profile_pane: bool,
    /// Indicates that the application should quit the main event loop
    /// as soon as possible
    pub quit: bool,
    pub focus: Focus,
    pub key_map_settings: bool,
    pub bottom_state: BottomState,
    pub bottom_edit_field: BottomEditField,
}

impl Global {
    pub fn new() -> Self {
        Self {
            profile_pane: false,
            quit: false,
            focus: Focus::Output,
            key_map_settings: false,
            bottom_state: BottomState::Nothing,
            bottom_edit_field: BottomEditField::new(),
        }
    }

    pub fn handle_key(&mut self, key_stroke: KeyStroke, key_map: &KeyMap) -> bool {
        if key_map.is_bound(key_stroke, action::TOGGLE_PROFILE_PANEL) {
            self.profile_pane = !self.profile_pane;
            true
        } else if key_map.is_bound(key_stroke, action::QUIT) {
            self.quit = true;
            true
        } else if key_map.is_bound(key_stroke, action::FOCUS_FORWARD) {
            self.focus = self.focus.next();
            true
        } else if key_map.is_bound(key_stroke, action::FOCUS_BACKWARD) {
            self.focus = self.focus.prev();
            true
        } else if key_map.is_bound(key_stroke, action::OPEN_KEY_MAP_SETTINGS) {
            self.key_map_settings = true;
            true
        } else {
            false
        }
    }
}
