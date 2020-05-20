use libc::*;
use ncurses::*;
use regex::Regex;
use std::error::Error;
use std::ffi::CString;
use std::fs::{File, read_to_string, create_dir_all};
use std::io::{stdin, Write};
use std::process::Command;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::env::var;

trait RenderItem {
    fn render(&self, row: Row, cursor_x: usize,
              selected: bool, focused: bool);
}

struct ItemList<Item> {
    items: Vec<Item>,
    cursor_x: usize,
    cursor_y: usize,
}

impl<T> Default for ItemList<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            cursor_x: 0,
            cursor_y: 0,
        }
    }
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

    fn handle_key(&mut self, key: char) {
        match key {
            's' => self.down(),
            'w' => self.up(),
            'd' => self.right(),
            'a' => self.left(),
            _ => {}
        }
    }

    fn render(&self, Rect {x, y, w, h}: Rect, focused: bool) {
        if h > 0 {
            // TODO(#16): word wrapping for long lines
            for (i, item) in self.items.iter().skip(self.cursor_y / h * h).enumerate().take_while(|(i, _)| *i < h) {
                item.render(Row {x: x, y: i + y, w: w}, self.cursor_x,
                            i == (self.cursor_y % h),
                            focused);
            }
        }
    }

    fn current_item(&self) -> &Item {
        &self.items[self.cursor_y]
    }
}

impl RenderItem for String {
    fn render(&self, Row {x, y, w} : Row, cursor_x: usize, selected: bool, focused: bool) {
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
            if focused {
                CURSOR_PAIR
            } else {
                UNFOCUSED_CURSOR_PAIR
            }
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
    fn render(&self, row : Row, cursor_x: usize, selected: bool, focused: bool) {
        let Row {x, y, w} = row;
        self.text.render(row, cursor_x, selected, focused);

        let cap_pair = if selected {
            if focused {
                MATCH_CURSOR_PAIR
            } else {
                UNFOCUSED_MATCH_CURSOR_PAIR
            }
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
const UNFOCUSED_CURSOR_PAIR: i16 = 3;
const MATCH_PAIR: i16 = 4;
const MATCH_CURSOR_PAIR: i16 = 5;
const UNFOCUSED_MATCH_CURSOR_PAIR: i16 = 6;

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

#[derive(Default)]
struct Profile {
    regex_list: ItemList<String>,
    cmd_list: ItemList<String>,
}

impl Profile {
    fn default_profile() -> Self {
        Self {
            regex_list: ItemList {
                items: vec![r"^(.*?):(\d+):".to_string()],
                cursor_x: 0,
                cursor_y: 0,
            },
            cmd_list: ItemList {
                items: vec!["vim +\\2 \\1".to_string(),
                            "emacs -nw +\\2 \\1".to_string()],
                cursor_x: 0,
                cursor_y: 0
            },
        }
    }

    fn from_file(file_path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut result = Profile::default();
        let input = read_to_string(file_path)?;
        for (i, line) in input.lines().map(|x| x.trim_start()).enumerate() {
            let fail = |message| {
                format!("{}:{}: {}", file_path.display(), i + 1, message)
            };

            if line.len() > 0 {
                let mut assign = line.split('=');
                let key   = assign.next().ok_or(fail("Key is not provided"))?.trim();
                let value = assign.next().ok_or(fail("Value is not provided"))?.trim();
                match key {
                    "regexs"        =>
                        result.regex_list.items.push(value.to_string()),
                    "cmds"          =>
                        result.cmd_list.items.push(value.to_string()),
                    // TODO(#49): cm crashes if current_regex or current_cmd from cm.conf is out-of-bound
                    //   I think we should simply clamp it to the allowed rage
                    "current_regex" => result.regex_list.cursor_y = value
                        .parse::<usize>()
                        .map_err(|_| fail("Not a number"))?,
                    "current_cmd"   => result.cmd_list.cursor_y = value
                        .parse::<usize>()
                        .map_err(|_| fail("Not a number"))?,
                    _               =>
                        Err(fail(&format!("Unknown key {}", key))).unwrap(),
                }
            }
        }

        Ok(result)
    }

    fn to_file<F: Write>(&self, stream: &mut F) -> Result<(), Box<dyn Error>> {
        for regex in self.regex_list.items.iter() {
            write!(stream, "regexs = {}\n", regex)?;
        }

        for cmd in self.cmd_list.items.iter() {
            write!(stream, "cmds = {}\n", cmd)?;
        }

        write!(stream, "current_regex = {}\n", self.regex_list.cursor_y)?;
        write!(stream, "current_cmd = {}\n", self.cmd_list.cursor_y)?;

        Ok(())
    }

    fn compile_current_regex(&self) -> Result<Regex, impl Error> {
        Regex::new(self.regex_list.current_item())
    }

    fn render_cmdline(&self, line: &Line) -> String {
        let mut cmdline = self.cmd_list.current_item().clone();
        for (i, cap) in line.caps.iter().enumerate() {
            cmdline = cmdline.replace(
                format!("\\{}", i + 1).as_str(),
                line.text.get(cap.clone()).unwrap_or(""))
        }
        cmdline
    }
}

fn handle_line_list_key(line_list: &mut ItemList<Line>, key: char, cmdline: &str) -> Result<(), Box<dyn Error>> {
    match key {
        '\n' => {
            // TODO(#47): endwin() on Enter in LineList looks like a total hack and it's unclear why it even works
            endwin();
            // TODO(#40): shell is not customizable
            // TODO(#50): cm doesn't say anything if the executed command has failed
            Command::new("sh")
                .stdin(File::open("/dev/tty")?)
                .arg("-c")
                .arg(cmdline)
                .spawn()?
                .wait_with_output()?;
        }
        key => line_list.handle_key(key)
    };

    Ok(())
}

#[derive(PartialEq)]
enum Focus {
    LineList,
    RegexList,
    CmdList
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::LineList  => Focus::RegexList,
            Focus::RegexList => Focus::CmdList,
            Focus::CmdList   => Focus::LineList,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = {
        const CONFIG_FILE_NAME: &'static str = "cm.conf";
        let xdg_config_dir = var("XDG_CONFIG_HOME").map(PathBuf::from);
        let home_config_dir = var("HOME").map(PathBuf::from).map(|x| x.join(".config"));
        xdg_config_dir.or(home_config_dir).map(|p| p.join(CONFIG_FILE_NAME))?
    };

    let mut profile = if config_path.exists() {
        Profile::from_file(&config_path)?
    } else {
        Profile::default_profile()
    };

    let re = profile.compile_current_regex()?;
    let mut line_list = ItemList::default();
    let mut line_text: String = String::new();
    let mut focus = Focus::RegexList;
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
    init_pair(UNFOCUSED_CURSOR_PAIR, COLOR_BLACK, COLOR_CYAN);
    init_pair(MATCH_PAIR, COLOR_YELLOW, COLOR_BLACK);
    init_pair(MATCH_CURSOR_PAIR, COLOR_RED, COLOR_WHITE);
    init_pair(UNFOCUSED_MATCH_CURSOR_PAIR, COLOR_BLACK, COLOR_CYAN);

    let mut quit = false;
    let mut profile_pane = false;
    while !quit {
        let cmdline = profile.render_cmdline(line_list.current_item());

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

            line_list.render(Rect { x: 0, y: 0, w: w, h: list_h},
                             focus == Focus::LineList);
            // TODO(#31): no way to switch regex
            // TODO(#32): no way to add new regex
            // TODO(#51): no way to delete a regex
            profile.regex_list.render(Rect { x: 0, y: list_h, w: w / 2, h: working_h - list_h},
                                      focus == Focus::RegexList);
            // TODO(#34): no way to add new cmd
            // TODO(#52): no way to delete a cmd
            profile.cmd_list.render(Rect { x: w / 2, y: list_h, w: w - w / 2, h: working_h - list_h},
                                    focus == Focus::CmdList);
        } else {
            line_list.render(Rect { x: 0, y: 0, w: w, h: h - 1 }, true);
        }

        if h <= 1 {
            render_status(0, "MAKE THE WINDOW BIGGER YOU FOOL!");
        } else {
            render_status(h - 1, &cmdline);
        }
        refresh();
        let key = getch() as u8 as char;
        match key {
            'e'  => profile_pane = !profile_pane,
            'q'  => quit = true,
            // TODO(#43): cm does not handle Shift+TAB to scroll backwards through the panels
            '\t' => focus = focus.next(),
            key  => if !profile_pane {
                handle_line_list_key(&mut line_list, key, &cmdline)?;
            } else {
                match focus {
                    Focus::LineList  => handle_line_list_key(&mut line_list, key, &cmdline)?,
                    Focus::RegexList => profile.regex_list.handle_key(key),
                    Focus::CmdList   => profile.cmd_list.handle_key(key),
                }
            }
        }
    }

    // TODO(#21): if application crashes it does not finalize the terminal
    endwin();

    config_path.parent().map(create_dir_all);
    profile.to_file(&mut File::create(config_path)?)?;

    Ok(())
}
