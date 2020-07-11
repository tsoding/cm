mod ui;

use libc::*;
use ncurses::*;
use os_pipe::{pipe, PipeReader};
use pcre2::bytes::Regex;
use std::env::var;
use std::error::Error;
use std::ffi::CString;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::{stdin, BufRead, BufReader, Write};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use ui::keycodes::*;
use ui::style::*;
use ui::*;

// TODO(#94): mark_nonblocking does not work on Windows
fn mark_nonblocking<Fd: AsRawFd>(fd: &mut Fd) {
    unsafe {
        let flags = libc::fcntl(fd.as_raw_fd(), F_GETFL, 0);
        libc::fcntl(fd.as_raw_fd(), F_SETFL, flags | O_NONBLOCK);
    }
}

struct LineList {
    list: ItemList,
    child: Option<(BufReader<PipeReader>, Child)>,
}

impl LineList {
    fn new() -> Self {
        Self {
            list: ItemList::new(),
            child: None,
        }
    }

    fn current_item(&self) -> Option<&str> {
        self.list.current_item()
    }

    fn render(&self, rect: Rect, focused: bool, regex_result: Option<Result<Regex, pcre2::Error>>) {
        self.list.render(rect, focused);

        let Rect { x, y, w, h } = rect;
        if h > 0 {
            // TODO(#16): word wrapping for long lines
            for (i, item) in self
                .list
                .items
                .iter()
                .skip(self.list.cursor_y / h * h)
                .enumerate()
                .take_while(|(i, _)| *i < h)
            {
                let selected = i == (self.list.cursor_y % h);

                let cap_pair = if selected {
                    if focused {
                        MATCH_CURSOR_PAIR
                    } else {
                        UNFOCUSED_MATCH_CURSOR_PAIR
                    }
                } else {
                    MATCH_PAIR
                };

                if let Some(Ok(regex)) = &regex_result {
                    // NOTE: we are ignoring any further potential
                    // capture matches (I don't like this term but
                    // that's what PCRE2 lib is calling it). For no
                    // particular reason. Just to simplify the
                    // implementation. Maybe in the future it will
                    // make sense.
                    let cap_mats = regex.captures_iter(item.as_bytes()).next();
                    if let Some(cap_mat) = cap_mats {
                        if let Ok(caps) = cap_mat {
                            // NOTE: we are skiping first cap because it contains the
                            // whole match which is not needed in our case
                            for j in 1..caps.len() {
                                if let Some(mat) = caps.get(j) {
                                    let start = usize::max(self.list.cursor_x, mat.start());
                                    let end = usize::min(self.list.cursor_x + w, mat.end());
                                    if start != end {
                                        mv((y + i) as i32, (start - self.list.cursor_x + x) as i32);
                                        attron(COLOR_PAIR(cap_pair));
                                        addstr(item.get(start..end).unwrap_or(""));
                                        attroff(COLOR_PAIR(cap_pair));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn refresh_child_output(&mut self) -> Result<bool, Box<dyn Error>> {
        if let Some(shell) = &std::env::args().nth(1) {
            // TODO(#102): cm does not warn the user when it kills the child process
            if let Some((_, child)) = &mut self.child {
                child.kill()?;
                child.wait()?;
                self.child = None;
            }

            // @shell
            let mut command = Command::new("sh");
            command.arg("-c");
            command.arg(shell);
            let (mut reader, writer) = pipe()?;
            let writer_clone = writer.try_clone()?;
            command.stdout(writer);
            command.stderr(writer_clone);
            let child = command.spawn()?;
            drop(command);

            self.list.cursor_y = 0;
            self.list.items.clear();
            self.list
                .items
                .push(format!("PID: {}, Command: {}", child.id(), shell));

            mark_nonblocking(&mut reader);
            let output = BufReader::new(reader);

            self.child = Some((output, child));

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn handle_key(
        &mut self,
        key: i32,
        cmdline_result: &Option<String>,
        global: &mut Global,
    ) -> Result<(), Box<dyn Error>> {
        if !global.handle_key(key) {
            match key {
                KEY_RETURN => {
                    if let Some(cmdline) = cmdline_result {
                        // TODO(#47): endwin() on Enter in LineList looks like a total hack and it's unclear why it even works
                        endwin();
                        // TODO(#40): shell is not customizable
                        //   Grep for @shell
                        // TODO(#50): cm doesn't say anything if the executed command has failed
                        Command::new("sh")
                            .stdin(File::open("/dev/tty")?)
                            .arg("-c")
                            .arg(cmdline)
                            .spawn()?
                            .wait_with_output()?;
                    }
                    // TODO(#105): print error in status line on enter when line does not match the regex
                }
                KEY_F5 => self.refresh_child_output().map(|_| ())?,
                key => self.list.handle_key(key),
            }
        }

        Ok(())
    }
}

#[derive(PartialEq)]
enum StringListState {
    Navigate,
    Editing { new: bool },
}

struct StringList {
    state: StringListState,
    list: ItemList,
    edit_field: EditField,
}

impl StringList {
    fn new() -> Self {
        Self {
            state: StringListState::Navigate,
            list: ItemList::new(),
            edit_field: EditField::new(),
        }
    }

    fn current_item(&self) -> Option<&str> {
        self.list.current_item()
    }

    fn render(&self, rect: Rect, focused: bool, global: &mut Global) {
        self.list.render(rect, focused);
        if let StringListState::Editing { .. } = self.state {
            let row = self.list.current_row(rect);
            self.edit_field.render(row);
            global.cursor_y = row.y as i32;
            global.cursor_x = (row.x + self.edit_field.cursor_x % row.w) as i32;
        }
    }

    fn handle_key(&mut self, key: i32, global: &mut Global) {
        match self.state {
            StringListState::Navigate => {
                if !global.handle_key(key) {
                    match key {
                        KEY_I => {
                            self.list.items.insert(self.list.cursor_y, String::new());
                            self.edit_field.buffer.clear();
                            self.edit_field.cursor_x = 0;
                            self.state = StringListState::Editing { new: true };
                            global.cursor_visible = true;
                        }
                        KEY_F2 => {
                            if let Some(item) = self.list.current_item() {
                                self.edit_field.cursor_x = item.len();
                                self.edit_field.buffer = String::from(item);
                                self.state = StringListState::Editing { new: false };
                                global.cursor_visible = true;
                            }
                        }
                        key => self.list.handle_key(key),
                    }
                }
            }
            StringListState::Editing { new } => match key {
                KEY_RETURN => {
                    self.state = StringListState::Navigate;
                    self.list.items[self.list.cursor_y] = self.edit_field.buffer.clone();
                    global.cursor_visible = false;
                }
                KEY_ESCAPE => {
                    self.state = StringListState::Navigate;
                    if new {
                        self.list.delete_current()
                    }
                    global.cursor_visible = false;
                }
                key => self.edit_field.handle_key(key),
            },
        }
    }
}

struct StatusLine {
    text: String,
}

impl StatusLine {
    fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    fn render(&self, y: usize) {
        attron(COLOR_PAIR(REGULAR_PAIR));
        mv(y as i32, 0);
        addstr(self.text.as_str());
        attroff(COLOR_PAIR(REGULAR_PAIR));
    }

    fn update(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

struct Profile {
    regex_list: StringList,
    cmd_list: StringList,
}

impl Profile {
    fn new() -> Self {
        Self {
            regex_list: StringList::new(),
            cmd_list: StringList::new(),
        }
    }

    fn from_file(file_path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut result = Profile::new();
        let input = read_to_string(file_path)?;
        let (mut regex_count, mut cmd_count) = (0, 0);
        for (i, line) in input.lines().map(|x| x.trim_start()).enumerate() {
            let fail = |message| format!("{}:{}: {}", file_path.display(), i + 1, message);

            if !line.is_empty() {
                let mut assign = line.split('=');
                let key = assign
                    .next()
                    .ok_or_else(|| fail("Key is not provided"))?
                    .trim();
                let value = assign
                    .next()
                    .ok_or_else(|| fail("Value is not provided"))?
                    .trim();
                match key {
                    "regexs" => {
                        regex_count += 1;
                        result.regex_list.list.items.push(value.to_string());
                    }
                    "cmds" => {
                        cmd_count += 1;
                        result.cmd_list.list.items.push(value.to_string());
                    }
                    "current_regex" => {
                        result.regex_list.list.cursor_y =
                            value.parse::<usize>().map_err(|_| fail("Not a number"))?
                    }
                    "current_cmd" => {
                        result.cmd_list.list.cursor_y =
                            value.parse::<usize>().map_err(|_| fail("Not a number"))?
                    }
                    _ => Err(fail(&format!("Unknown key {}", key))).unwrap(),
                }
            }
        }

        // NOTE: regex_count-1 converts value from count to 0-based index
        if result.regex_list.list.cursor_y > regex_count - 1 {
            result.regex_list.list.cursor_y = regex_count - 1;
        }

        // NOTE: cmd_count-1 converts value from count to 0-based index
        if result.cmd_list.list.cursor_y > cmd_count - 1 {
            result.cmd_list.list.cursor_y = cmd_count - 1;
        }

        Ok(result)
    }

    fn to_file<F: Write>(&self, stream: &mut F) -> Result<(), Box<dyn Error>> {
        for regex in self.regex_list.list.items.iter() {
            writeln!(stream, "regexs = {}", regex)?;
        }

        for cmd in self.cmd_list.list.items.iter() {
            writeln!(stream, "cmds = {}", cmd)?;
        }

        writeln!(stream, "current_regex = {}", self.regex_list.list.cursor_y)?;
        writeln!(stream, "current_cmd = {}", self.cmd_list.list.cursor_y)?;

        Ok(())
    }

    fn current_regex(&self) -> Option<Result<Regex, pcre2::Error>> {
        match self.regex_list.state {
            StringListState::Navigate => self.regex_list.current_item().map(|s| Regex::new(&s)),
            StringListState::Editing { .. } => Some(Regex::new(&self.regex_list.edit_field.buffer)),
        }
    }

    fn current_cmd(&self) -> Option<String> {
        match self.cmd_list.state {
            StringListState::Navigate => self.cmd_list.current_item().map(String::from),
            StringListState::Editing { .. } => Some(self.cmd_list.edit_field.buffer.clone()),
        }
    }

    fn initial() -> Self {
        let mut result = Self::new();
        result
            .regex_list
            .list
            .items
            .push(r"(\/?\b.*?):(\d+):".to_string());
        result.cmd_list.list.items.push("vim +\\2 \\1".to_string());
        result
            .cmd_list
            .list
            .items
            .push("emacs -nw +\\2 \\1".to_string());
        result
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Focus {
    Lines = 0,
    Regexs = 1,
    Cmds = 2,
}

const FOCUS_COUNT: usize = 3;

impl Focus {
    fn from_number(n: usize) -> Option<Focus> {
        match n {
            0 => Some(Focus::Lines),
            1 => Some(Focus::Regexs),
            2 => Some(Focus::Cmds),
            _ => None,
        }
    }

    fn next(self) -> Self {
        Focus::from_number((self as usize + 1) % FOCUS_COUNT).unwrap()
    }

    fn steps_to(self, target: Focus) -> usize {
        let a = self as usize;
        let b = target as usize;
        if a <= b {
            b - a
        } else {
            (b + FOCUS_COUNT) - a
        }
    }
}

struct Global {
    profile_pane: bool,
    quit: bool,
    focus: Focus,
    cursor_visible: bool,
    cursor_x: i32,
    cursor_y: i32,
}

impl Global {
    fn handle_key(&mut self, key: i32) -> bool {
        match key {
            KEY_E => {
                self.profile_pane = !self.profile_pane;
                true
            }
            KEY_Q => {
                self.quit = true;
                true
            }
            KEY_TAB => {
                self.focus = self.focus.next();
                true
            }
            _ => false,
        }
    }
}

fn render_cmdline(line: &str, cmd: &str, regex: Regex) -> Option<String> {
    regex
        .captures_iter(line.as_bytes())
        .next()
        .and_then(|cap_mat| {
            let mut result = cmd.to_string();
            if let Ok(caps) = cap_mat {
                for i in 1..caps.len() {
                    if let Some(mat) = caps.get(i) {
                        let mut match_str =
                            line.get(mat.start()..mat.end()).unwrap_or("").to_string();

                        if match_str.contains('"') {
                            match_str = match_str.replace('"', "\\\"")
                        }
                        if match_str.contains(' ') {
                            match_str = format!("\"{}\"", match_str)
                        }

                        result = result.replace(format!("\\{}", i).as_str(), &match_str)
                    }
                }
            }
            Some(result)
        })
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = {
        const CONFIG_FILE_NAME: &str = "cm.conf";
        let xdg_config_dir = var("XDG_CONFIG_HOME").map(PathBuf::from);
        let home_config_dir = var("HOME").map(PathBuf::from).map(|x| x.join(".config"));
        xdg_config_dir
            .or(home_config_dir)
            .map(|p| p.join(CONFIG_FILE_NAME))?
    };

    let mut profile = if config_path.exists() {
        Profile::from_file(&config_path)?
    } else {
        Profile::initial()
    };

    let mut global = Global {
        quit: false,
        profile_pane: false,
        focus: Focus::Regexs,
        cursor_x: 0,
        cursor_y: 0,
        cursor_visible: false,
    };

    let mut line_list = LineList::new();
    let mut status_line = StatusLine::new();

    if !line_list.refresh_child_output()? {
        line_list.list.items = stdin().lock().lines().collect::<Result<Vec<String>, _>>()?;
    }

    // NOTE: stolen from https://stackoverflow.com/a/44884859
    // TODO(#3): the terminal redirection is too hacky
    let tty_path = CString::new("/dev/tty")?;
    let fopen_mode = CString::new("r+")?;
    let file = unsafe { fopen(tty_path.as_ptr(), fopen_mode.as_ptr()) };
    let screen = newterm(None, file, file);
    set_term(screen);

    keypad(stdscr(), true);
    timeout(0);

    init_style();

    while !global.quit {
        // BEGIN CMDLINE RENDER SECTION //////////////////////////////
        let cmdline = match (
            profile.current_regex(),
            profile.current_cmd(),
            line_list.current_item(),
        ) {
            (None, _, _) => {
                if global.profile_pane {
                    match global.focus.steps_to(Focus::Regexs) {
                        0 => status_line.update("No regex (press I to insert a regex)"),
                        x => status_line.update(&format!(
                            "No regex (press TAB {} times to focus on regex panel)",
                            x
                        )),
                    }
                } else {
                    status_line.update("No regex (press E to add a regex)")
                }
                None
            }
            (Some(Err(err)), _, _) => {
                status_line.update(&err.to_string());
                None
            }
            (Some(Ok(_)), None, _) => {
                if global.profile_pane {
                    match global.focus.steps_to(Focus::Cmds) {
                        0 => status_line.update("No command (press I to insert a command)"),
                        x => status_line.update(&format!(
                            "No command (press TAB {} times to focus on command panel)",
                            x
                        )),
                    }
                } else {
                    status_line.update("No command (press E to add a command)");
                }
                None
            }
            (Some(Ok(_)), Some(_), None) => {
                status_line.update("No line selected");
                None
            }
            (Some(Ok(regex)), Some(cmd), Some(line)) => match render_cmdline(line, &cmd, regex) {
                Some(cmdline) => {
                    status_line.update(&cmdline);
                    Some(cmdline)
                }
                None => {
                    status_line.update(
                        "No match (line doesn't match current regex, nothing happens on ENTER)",
                    );
                    None
                }
            },
        };
        // END CMDLINE RENDER SECTION //////////////////////////////

        // BEGIN RENDER SECTION //////////////////////////////
        // TODO(#95): Don't rerender the state of the app if nothing changed
        //   After introducing async input we are rerendering the whole application
        //   on each iteration of even loop. And the rendering operation is pretty
        //   expensive by itself.
        let (w, h) = {
            let mut x: i32 = 0;
            let mut y: i32 = 0;
            getmaxyx(stdscr(), &mut y, &mut x);
            (x as usize, y as usize)
        };

        erase();

        if h >= 1 {
            status_line.render(h - 1);
        }

        if global.profile_pane {
            let working_h = h - 1;
            let list_h = working_h / 3 * 2;

            line_list.render(
                Rect {
                    x: 0,
                    y: 0,
                    w,
                    h: list_h,
                },
                global.focus == Focus::Lines,
                profile.current_regex(),
            );
            profile.regex_list.render(
                Rect {
                    x: 0,
                    y: list_h,
                    w: w / 2,
                    h: working_h - list_h,
                },
                global.focus == Focus::Regexs,
                &mut global,
            );
            profile.cmd_list.render(
                Rect {
                    x: w / 2,
                    y: list_h,
                    w: w - w / 2,
                    h: working_h - list_h,
                },
                global.focus == Focus::Cmds,
                &mut global,
            );
        } else {
            line_list.render(
                Rect {
                    x: 0,
                    y: 0,
                    w,
                    h: h - 1,
                },
                true,
                profile.current_regex(),
            );
        }

        curs_set(if global.cursor_visible {
            ncurses::CURSOR_VISIBILITY::CURSOR_VISIBLE
        } else {
            ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE
        });
        mv(global.cursor_y, global.cursor_x);

        refresh();
        // END RENDER SECTION //////////////////////////////

        // BEGIN INPUT SECTION //////////////////////////////
        // TODO(#43): cm does not handle Shift+TAB to scroll backwards through the panels
        let key = getch();
        let profile_pane = global.profile_pane;
        if !profile_pane {
            line_list.handle_key(key, &cmdline, &mut global)?;
        } else {
            match global.focus {
                Focus::Lines => line_list.handle_key(key, &cmdline, &mut global)?,
                Focus::Regexs => profile.regex_list.handle_key(key, &mut global),
                Focus::Cmds => profile.cmd_list.handle_key(key, &mut global),
            }
        }
        // END INPUT SECTION //////////////////////////////

        // BEGIN ASYNC CHILD OUTPUT SECTION //////////////////////////////
        if let Some((reader, child)) = &mut line_list.child {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => line_list.list.items.push(line.clone()),
                    _ => break,
                }
            }

            if let Some(status) = child.try_wait()? {
                match status.code() {
                    Some(code) => line_list.list.items.push(format!(
                        "-- Execution Finished with status code: {} --",
                        code
                    )),
                    None => line_list
                        .list
                        .items
                        .push("-- Execution Terminated by a signal --".to_string()),
                }
                line_list.child = None
            }
        }
        // END ASYNC CHILD OUTPUT SECTION //////////////////////////////

        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // TODO(#21): if application crashes it does not finalize the terminal
    endwin();

    config_path.parent().map(create_dir_all);
    profile.to_file(&mut File::create(config_path)?)?;

    Ok(())
}
