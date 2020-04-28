use ncurses::*;
use regex::Regex;
use std::io::stdin;
use libc::*;
use std::ffi::CString;
use std::process::Command;
use std::fs::File;

const REGULAR_PAIR: i16 = 1;
const CURSOR_PAIR: i16 = 2;

fn render_list(lines: &Vec<String>, cursor: usize) {
    // TODO(#1): captured regexp groups are not highlighted
    for (i, line) in lines.iter().enumerate() {
        mv(i as i32, 0);
        let pair = if i == cursor { CURSOR_PAIR } else { REGULAR_PAIR };
        attron(COLOR_PAIR(pair));
        printw(line.as_str());
        attroff(COLOR_PAIR(pair));
    }
}

fn main() -> Result<(), String> {
    // TODO(#2): regexp is not customizable
    let re = Regex::new(r"^(.*?):(\d+):").map_err(|e| e.to_string())?;

    let mut lines: Vec<String> = Vec::new();
    let mut cursor: usize = 0;
    let mut line: String = String::new();
    while stdin().read_line(&mut line).map_err(|e| e.to_string())? > 0 {
        lines.push(line.clone());
        line.clear();
    }

    // NOTE: stolen from https://stackoverflow.com/a/44884859
    // TODO(#3): the terminal redirection is too hacky
    let tty_path = CString::new("/dev/tty").map_err(|e| e.to_string())?;
    let fopen_mode = CString::new("r+").map_err(|e| e.to_string())?;
    let file = unsafe { fopen(tty_path.as_ptr(), fopen_mode.as_ptr()) };
    let screen = newterm(None, file, file);
    set_term(screen);

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(CURSOR_PAIR, COLOR_BLACK, COLOR_WHITE);

    loop {
        erase();
        render_list(&lines, cursor);
        refresh();
        match getch() {
            115 => if cursor + 1 < lines.len() { cursor += 1 },
            119 => if cursor > 0 { cursor -= 1 },
            10 => {
                endwin();
                for cap in re.captures_iter(lines[cursor].as_str()) {
                    // TODO(#6): the program does not go back after exiting vim
                    Command::new("vim")
                        .stdin(File::open("/dev/tty").unwrap())
                        .arg(format!("+{}", &cap[2]))
                        .arg(&cap[1])
                        .spawn().map_err(|e| e.to_string())?
                        .wait_with_output().map_err(|e| e.to_string())? ;
                }
            }
            _ => {},
        }
    }

    endwin();

    Ok(())
}
