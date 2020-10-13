use super::*;
use pcre2::bytes::Regex;

#[derive(PartialEq, Clone, Copy)]
pub enum Focus {
    Output = 0,
    Regexs = 1,
    Cmds = 2,
    Shell = 3,
}

const FOCUS_COUNT: usize = 4;

impl Focus {
    pub fn from_number(n: usize) -> Option<Focus> {
        match n {
            0 => Some(Focus::Output),
            1 => Some(Focus::Regexs),
            2 => Some(Focus::Cmds),
            3 => Some(Focus::Shell),
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
    pub cursor: Cursor,
    /// user_provided_cmdline is the line provided by the user through the CLI of cm:
    /// `cm <user_provided_cmdline>`
    pub user_provided_cmdline: Option<String>,
    pub search_regex: Option<Regex>,
    pub shell: String,
}

impl Global {
    pub fn new(user_provided_cmdline: Option<String>) -> Self {
        Self {
            profile_pane: false,
            quit: false,
            focus: Focus::Output,
            key_map_settings: false,
            bottom_state: BottomState::Nothing,
            bottom_edit_field: BottomEditField::new(),
            cursor: Cursor::new(),
            user_provided_cmdline,
            search_regex: None,
            shell: String::new(),
        }
    }

    pub fn handle_key(&mut self, key_stroke: KeyStroke, key_map: &KeyMap) -> bool {
        match key_map.check_bound(key_stroke) {
            action::TOGGLE_PROFILE_PANEL => {
                self.profile_pane = !self.profile_pane;
                true
            }
            action::QUIT => {
                self.quit = true;
                true
            }
            action::FOCUS_FORWARD => {
                self.focus = self.focus.next();
                true
            }
            action::FOCUS_BACKWARD => {
                self.focus = self.focus.prev();
                true
            }
            action::OPEN_KEY_MAP_SETTINGS => {
                self.key_map_settings = true;
                true
            }
            action::START_SEARCH => {
                if self.bottom_state == BottomState::Nothing {
                    // TODO(#160): cm search does not support jumping to next/previous matches
                    self.bottom_state = BottomState::Search;
                    self.bottom_edit_field
                        .activate(&mut self.cursor, String::new());
                    true
                } else {
                    false
                }
            }
            action::EDIT_CMDLINE => {
                if self.bottom_state == BottomState::Nothing {
                    self.bottom_state = BottomState::Cmdline;
                    self.bottom_edit_field.activate(
                        &mut self.cursor,
                        self.user_provided_cmdline
                            .clone()
                            .unwrap_or_else(String::new),
                    );
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
