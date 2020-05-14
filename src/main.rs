use libc::*;
use ncurses::*;
use regex::Regex;
use std::error::Error;
use std::ffi::CString;
use std::fs::File;
use std::io::stdin;
use std::process::Command;
use std::ops::Range;

trait RenderItem {
    fn render(&self, row: Row, cursor_x: usize, selected: bool);
}

struct ItemList<Item> {
    items: Vec<Item>,
    cursor_x: usize,
    cursor_y: usize,
}

impl<Item> ItemList<Item> where Item: RenderItem {
    fn up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1
        }
    }

    fn down(&mut self) {
        if self.cursor_y + 1 < self.items.len() {
            self.cursor_y += 1;
        }
    }

    fn left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    fn right(&mut self) {
        self.cursor_x += 1;
    }

    fn render(&self, Rect {x, y, w, h}: Rect) {
        if h > 0 {
            // TODO(#16): word wrapping for long lines
            for (i, item) in self.items.iter().skip(self.cursor_y / h * h).enumerate().take_while(|(i, _)| *i < h) {
                item.render(Row {x: x, y: i + y, w: w}, self.cursor_x, i == (self.cursor_y % h));
            }
        }
    }
}

impl RenderItem for String {
    fn render(&self, Row {x, y, w} : Row, cursor_x: usize, selected: bool) {
        let line_to_render = {
            let mut line_to_render = self
                .trim_end()
                .get(cursor_x..)
                .unwrap_or("")
                .to_string();
            let n = line_to_render.len();
            if n < w {
                for _ in 0..(w - n) {
                    line_to_render.push(' ');
                }
            }
            line_to_render
        };

        mv(y as i32, x as i32);
        let pair = if selected {
            CURSOR_PAIR
        } else {
            REGULAR_PAIR
        };
        attron(COLOR_PAIR(pair));
        addstr(&line_to_render);
        attroff(COLOR_PAIR(pair));
    }
}

#[derive(Debug)]
struct Line {
    text: String,
    caps: Vec<Range<usize>>,
}

impl Line {
    fn from_string(text: &str) -> Self {
        Self {
            text: String::from(text),
            caps: Vec::new(),
        }
    }
}

impl RenderItem for Line {
    fn render(&self, row : Row, cursor_x: usize, selected: bool) {
        let Row {x, y, w} = row;
        self.text.render(row, cursor_x, selected);

        let cap_pair = if selected {
            MATCH_CURSOR_PAIR
        } else {
            MATCH_PAIR
        };

        for cap in &self.caps {
            let start = usize::max(cursor_x, cap.start);
            let end = usize::min(cursor_x + w, cap.end);
            if start != end {
                mv(y as i32, (start - cursor_x + x) as i32);
                attron(COLOR_PAIR(cap_pair));
                addstr(self.text.get(start..end).unwrap_or(""));
                attroff(COLOR_PAIR(cap_pair));
            }
        }
    }
}

const REGULAR_PAIR: i16 = 1;
const CURSOR_PAIR: i16 = 2;
const MATCH_PAIR: i16 = 3;
const MATCH_CURSOR_PAIR: i16 = 4;

fn render_status(y: usize, text: &str) {
    mv(y as i32, 0);
    addstr(text);
}

struct Rect {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

struct Row {
    x: usize,
    y: usize,
    w: usize,
}

struct Profile {
    regex_list: ItemList<String>,
    cmd_list: ItemList<String>,
}

impl Profile {
    fn current_regex(&self) -> Result<Regex, impl Error> {
        Regex::new(self.regex_list.items[self.regex_list.cursor_y].as_str())
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            regex_list: ItemList::<String> {
                items: vec![r"^(.*?):(\d+):".to_string()],
                cursor_x: 0,
                cursor_y: 0,
            },
            cmd_list: ItemList::<String> {
                items: vec!["$EDITOR +\\2 \\1".to_string()],
                cursor_x: 0,
                cursor_y: 0
            },
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // TODO(#30): profile is not saved/loaded to/from file system
    let profile = Profile::default();
    let re = profile.current_regex()?;
    let mut line_list = ItemList::<Line> {
        items: Vec::new(),
        cursor_x: 0,
        cursor_y: 0,
    };
    let mut line_text: String = String::new();
    while stdin().read_line(&mut line_text)? > 0 {
        let caps = re.captures_iter(line_text.as_str()).next();
        let mut line = Line::from_string(line_text.as_str());

        for cap in caps {
            // NOTE: we are skiping first cap because it contains the
            // whole match which is not needed in our case
            for mat_opt in cap.iter().skip(1) {
                if let Some(mat) = mat_opt {
                    line.caps.push(mat.into())
                }
            }
        }

        line_list.items.push(line);
        line_text.clear();
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
    init_pair(MATCH_PAIR, COLOR_YELLOW, COLOR_BLACK);
    init_pair(MATCH_CURSOR_PAIR, COLOR_RED, COLOR_WHITE);

    let mut quit = false;
    let mut profile_pane = false;
    while !quit {
        let mut cmdline = profile.cmd_list.items[profile.cmd_list.cursor_y].clone();
        for (i, cap) in line_list.items[line_list.cursor_y].caps.iter().enumerate() {
            cmdline = cmdline.replace(
                format!("\\{}", i + 1).as_str(),
                line_list.items[line_list.cursor_y]
                    .text.get(cap.clone())
                    .unwrap_or(""))
        }

        let (w, h) = {
            let mut x: i32 = 0;
            let mut y: i32 = 0;
            getmaxyx(stdscr(), &mut y, &mut x);
            (x as usize, y as usize)
        };

        erase();

        if profile_pane {
            let working_h = h - 1;
            let list_h = working_h / 3 * 2;

            line_list.render(Rect { x: 0, y: 0, w: w, h: list_h});
            // TODO(#31): no way to switch regex
            // TODO(#32): no way to add new regex
            profile.regex_list.render(Rect { x: 0, y: list_h, w: w / 2, h: working_h - list_h});
            // TODO(#33): no way to switch cmd
            // TODO(#34): no way to add new cmd
            profile.cmd_list.render(Rect { x: w / 2, y: list_h, w: w - w / 2, h: working_h - list_h});
        } else {
            line_list.render(Rect { x: 0, y: 0, w: w, h: h - 1 });
        }

        if h <= 1 {
            render_status(0, "MAKE THE WINDOW BIGGER YOU FOOL!");
        } else {
            render_status(h - 1, &cmdline);
        }
        refresh();
        match getch() as u8 as char {
            's'  => line_list.down(),
            'w'  => line_list.up(),
            'd'  => line_list.right(),
            'a'  => line_list.left(),
            'e'  => profile_pane = !profile_pane,
            '\n' => {
                endwin();
                Command::new("sh")
                    .stdin(File::open("/dev/tty")?)
                    .arg("-c")
                    .arg(cmdline)
                    .spawn()?
                    .wait_with_output()?;
            }
            'q' => quit = true,
            _ => {}
        }
    }

    // TODO(#21): if application crashes it does not finalize the terminal
    endwin();
    Ok(())
}
