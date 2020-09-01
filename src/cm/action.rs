pub type Type = usize;

// TODO(#145): Separate Delete Character and Delete Item actions
pub const UP: Type = 0;
pub const DOWN: Type = 1;
pub const LEFT: Type = 2;
pub const RIGHT: Type = 3;
pub const HOME: Type = 4;
pub const INSERT_AFTER_ITEM: Type = 5;
pub const INSERT_BEFORE_ITEM: Type = 6;
pub const DELETE: Type = 7;
pub const BACK_DELETE: Type = 8;
pub const EDIT_ITEM: Type = 9;
pub const DUP_AFTER_ITEM: Type = 10;
pub const DUP_BEFORE_ITEM: Type = 11;
pub const TOGGLE_PROFILE_PANEL: Type = 12;
pub const QUIT: Type = 13;
pub const FOCUS_FORWARD: Type = 14;
pub const FOCUS_BACKWARD: Type = 15;
pub const ACCEPT: Type = 16;
pub const CANCEL: Type = 17;
pub const RUN: Type = 18;
pub const RUN_INTO_ITSELF: Type = 19;
pub const RERUN: Type = 20;
pub const BACK: Type = 21;
pub const NEXT_MATCH: Type = 22;
pub const PREV_MATCH: Type = 23;
pub const EDIT_CMDLINE: Type = 24;
pub const OPEN_KEY_MAP_SETTINGS: Type = 25;
pub const START_SEARCH: Type = 26;
pub const JUMP_TO_START: Type = 27;
pub const JUMP_TO_END: Type = 28;
pub const NEXT_SEARCH_MATCH: Type = 29;
pub const PREV_SEARCH_MATCH: Type = 30;
pub const EDIT_SHELL: Type = 31;
pub const LEN: usize = 32;

pub const NAMES: [&str; LEN] = [
    "up",
    "down",
    "left",
    "right",
    "home",
    "insert_after_item",
    "insert_before_item",
    "delete",
    "back_delete",
    "edit_item",
    "dup_after_item",
    "dup_before_item",
    "toggle_profile_panel",
    "quit",
    "focus_forward",
    "focus_backward",
    "accept",
    "cancel",
    "run",
    "run_into_itself",
    "rerun",
    "back",
    "next_match",
    "prev_match",
    "edit_cmdline",
    "open_key_map_settings",
    "start_search",
    "jump_to_start",
    "jump_to_end",
    "next_search_match",
    "prev_search_match",
    "edit_shell",
];

pub fn from_str(s: &str) -> Result<Type, String> {
    for (action, name) in NAMES.iter().enumerate() {
        if *name == s {
            return Ok(action);
        }
    }

    Err(format!("Unknown action `{}`", s))
}
