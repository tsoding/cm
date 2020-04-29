use libc::*;
use ncurses::*;
use regex::Regex;
use std::error::Error;
use std::ffi::CString;
use std::fs::File;
use std::io::stdin;
use std::process::Command;

const REGULAR_PAIR: i16 = 1;
const CURSOR_PAIR: i16 = 2;

fn render_list(lines: &[String], cursor: usize) {
    // TODO(#1): captured regexp groups are not highlighted
    for (i, line) in lines.iter().enumerate() {
        mv(i as i32, 0);
        let pair = if i == cursor {
            CURSOR_PAIR
        } else {
            REGULAR_PAIR
        };
        attron(COLOR_PAIR(pair));
        addstr(&line);
        attroff(COLOR_PAIR(pair));
    }
}

/// Read a key press as a char
fn get_char() -> char {
    getch() as u8 as char
}

fn main() -> Result<(), Box<dyn Error>> {
    // TODO(#2): regexp is not customizable
    let re = Regex::new(r"^(.*?):(\d+):")?;

    let mut lines: Vec<String> = Vec::new();
    let mut cursor: usize = 0;
    let mut line: String = String::new();
    while stdin().read_line(&mut line)? > 0 {
        lines.push(line.clone());
        line.clear();
    }

    // NOTE: stolen from https://stackoverflow.com/a/44884859
    // TODO(#3): the terminal redirection is too hacky
    let tty_path = CString::new("/dev/tty")?;
    let fopen_mode = CString::new("r+")?;
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
        match get_char() {
            's' => {
                if cursor + 1 < lines.len() {
                    cursor += 1;
                } else {
                    cursor = 0;
                }
            }
            'w' => {
                if cursor > 0 {
                    cursor -= 1;
                } else {
                    cursor = lines.len() - 1;
                }
            }
            '\n' => {
                endwin();
                for cap in re.captures_iter(lines[cursor].as_str()) {
                    // TODO(#6): the program does not go back after exiting vim
                    Command::new("vim")
                        .stdin(File::open("/dev/tty")?)
                        .arg(format!("+{}", &cap[2]))
                        .arg(&cap[1])
                        .spawn()?
                        .wait_with_output()?;
                }
            }
            'q' => break,
            _ => {}
        }
    }

    endwin();
    Ok(())
}
