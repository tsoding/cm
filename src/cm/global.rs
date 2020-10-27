use super::*;
use pcre2::bytes::Regex;

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Output,
    Regexs,
    Cmds,
}

#[derive(PartialEq)]
pub enum BottomState {
    Nothing,
    Cmdline,
    Search,
}

pub struct Global {
    /// Indicates that the application should quit the main event loop
    /// as soon as possible
    pub quit: bool,
    pub mode: Mode,
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
            quit: false,
            mode: Mode::Output,
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
        if key_map.is_bound(key_stroke, action::TOGGLE_PROFILE_PANEL) {
            // TODO: remove action::TOGGLE_PROFILE_PANEL
            true
        } else if key_map.is_bound(key_stroke, action::QUIT) {
            self.quit = true;
            true
        } else if key_map.is_bound(key_stroke, action::FOCUS_FORWARD) {
            // TODO: remove action::FOCUS_FORWARD
            true
        } else if key_map.is_bound(key_stroke, action::FOCUS_BACKWARD) {
            // TODO: remove action::FOCUS_BACKWARD
            true
        } else if key_map.is_bound(key_stroke, action::OPEN_KEY_MAP_SETTINGS) {
            self.key_map_settings = true;
            true
        } else if key_map.is_bound(key_stroke, action::REGEXS_MODE) {
            if self.mode == Mode::Regexs {
                self.mode = Mode::Output;
            } else {
                self.mode = Mode::Regexs;
            }
            true
        } else if key_map.is_bound(key_stroke, action::CMDS_MODE) {
            if self.mode == Mode::Cmds {
                self.mode = Mode::Output;
            } else {
                self.mode = Mode::Cmds;
            }
            true
        } else if self.bottom_state == BottomState::Nothing
            && key_map.is_bound(key_stroke, action::START_SEARCH)
        {
            // TODO(#160): cm search does not support jumping to next/previous matches
            self.bottom_state = BottomState::Search;
            self.bottom_edit_field
                .activate(&mut self.cursor, String::new());
            true
        } else if self.bottom_state == BottomState::Nothing
            && key_map.is_bound(key_stroke, action::EDIT_CMDLINE)
        {
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
}
