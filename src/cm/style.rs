use ncurses::*;

pub const REGULAR_PAIR: i16 = 1;
pub const CURSOR_PAIR: i16 = 2;
pub const UNFOCUSED_CURSOR_PAIR: i16 = 3;
pub const MATCH_PAIR: i16 = 4;
pub const MATCH_CURSOR_PAIR: i16 = 5;
pub const UNFOCUSED_MATCH_CURSOR_PAIR: i16 = 6;
pub const STATUS_ERROR_PAIR: i16 = 7;

pub fn init_style() {
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(CURSOR_PAIR, COLOR_BLACK, COLOR_WHITE);
    init_pair(UNFOCUSED_CURSOR_PAIR, COLOR_BLACK, COLOR_CYAN);
    init_pair(MATCH_PAIR, COLOR_YELLOW, COLOR_BLACK);
    init_pair(MATCH_CURSOR_PAIR, COLOR_RED, COLOR_WHITE);
    init_pair(UNFOCUSED_MATCH_CURSOR_PAIR, COLOR_BLACK, COLOR_CYAN);
    init_pair(STATUS_ERROR_PAIR, COLOR_RED, COLOR_BLACK);
}
