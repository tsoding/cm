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
    let (_x, y) = {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        getmaxyx(stdscr(), &mut y, &mut x);
        (x as usize, y as usize)
    };

    // TODO(#1): captured regexp groups are not highlighted
    // TODO: word wrapping for long lines
    for (i, line) in lines.iter()
                          .skip(cursor / y * y)
                          .enumerate()
                          .take_while(|(i, _)| *i < y) {
        mv(i as i32, 0);
        // TODO: cursor is not visible when the line is empty
        let pair = if i == (cursor % y) {
            CURSOR_PAIR
        } else {
            REGULAR_PAIR
        };
        attron(COLOR_PAIR(pair));
        addstr(&line);
        attroff(COLOR_PAIR(pair));
    }
}

/// Return a Regex pattern or use the default one
fn get_pattern() -> Result<Regex, impl Error> {
    let mut args = std::env::args();

    while let Some(arg) = args.next() {
        if arg == "--pattern" {
            if let Some(pattern) = args.next() {
                return Regex::new(&pattern);
            }
        }
    }

    Regex::new(r"^(.*?):(\d+):")
}

fn main() -> Result<(), Box<dyn Error>> {
    let re = get_pattern()?;

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

    let mut quit = false;
    while !quit {
        erase();
        render_list(&lines, cursor);
        refresh();
        match getch() as u8 as char {
            's' => if cursor + 1 < lines.len()  { cursor += 1; }
            'w' => if cursor > 0                { cursor -= 1; }
            '\n' => {
                endwin();
                for cap in re.captures_iter(lines[cursor].as_str()) {
                    Command::new("vim")
                        .stdin(File::open("/dev/tty")?)
                        .arg(format!("+{}", &cap[2]))
                        .arg(&cap[1])
                        .spawn()?
                        .wait_with_output()?;
                }
            }
            'q' => quit = true,
            _ => {}
        }
    }

    endwin();
    Ok(())
}
