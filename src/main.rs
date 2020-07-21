mod ui;

use libc::*;
use ncurses::*;
use os_pipe::{pipe, PipeReader};
use pcre2::bytes::Regex;
use std::env::var;
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
    lists: Vec<ItemList>,
    /// currently running process that generates data for LineList.
    /// See [LineList::poll_cmdline_output](struct.LineList.html#method.poll_cmdline_output)
    child: Option<(BufReader<PipeReader>, Child)>,
    /// user_provided_cmdline is the line provided by the user through the CLI of cm:
    /// `cm <user_provided_cmdline>`
    user_provided_cmdline: Option<String>,
}

impl LineList {
    fn new(user_provided_cmdline: Option<String>) -> Self {
        Self {
            lists: Vec::<ItemList>::new(),
            child: None,
            user_provided_cmdline,
        }
    }

    fn current_item(&self) -> Option<&str> {
        self.lists.last().and_then(|x| x.current_item())
    }

    fn render(&self, rect: Rect, focused: bool, regex_result: Option<Result<Regex, pcre2::Error>>) {
        if let Some(list) = self.lists.last() {
            list.render(rect, focused);

            let Rect { x, y, w, h } = rect;
            if h > 0 {
                // TODO(#16): word wrapping for long lines
                for (i, item) in list
                    .items
                    .iter()
                    .skip(list.cursor_y / h * h)
                    .enumerate()
                    .take_while(|(i, _)| *i < h)
                {
                    let selected = i == (list.cursor_y % h);

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
                                        let start = usize::max(list.cursor_x, mat.start());
                                        let end = usize::min(list.cursor_x + w, mat.end());
                                        if start != end {
                                            mv((y + i) as i32, (start - list.cursor_x + x) as i32);
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
    }

    fn run_cmdline(&mut self, cmdline: String) {
        // TODO(#102): cm does not warn the user when it kills the child process
        if let Some((_, child)) = &mut self.child {
            child
                .kill()
                .expect("Could not kill the currently running child process");
            child
                .wait()
                .expect("Error waiting for currently running child process");
            self.child = None;
        }

        // @ref(#40) customize the argument of Command::new()
        let mut command = Command::new("sh");
        command.arg("-c");
        command.arg(cmdline.clone());
        let (mut reader, writer) =
            pipe().expect("Could not create a pipe for collecting output from a child process");
        let writer_clone = writer
            .try_clone()
            .expect("Could not clone the pipe for collecting output from a child process");
        command.stdout(writer);
        command.stderr(writer_clone);
        // @ref(#40) this part should fail if the user provided
        // non-existing shell. So should probably do not unwrap it and
        // properly report the fail somehow without crashing the app.
        let child = command.spawn().expect("Could not spawn a child process");
        drop(command);

        let mut new_list = ItemList::new();
        new_list.items.push(format!(
            "PID: {}, Command: {}",
            child.id(),
            cmdline.as_str()
        ));
        self.lists.push(new_list);

        mark_nonblocking(&mut reader);
        let output = BufReader::new(reader);

        self.child = Some((output, child));
    }

    fn run_user_provided_cmdline(&mut self) {
        if let Some(cmdline) = self.user_provided_cmdline.clone() {
            self.run_cmdline(cmdline)
        }
    }

    /// Polls changes from the currently running child (see
    /// [LineList::run_cmdline](struct.LineList.html#method.run_cmdline),
    /// [LineList::child](struct.LineList.html#structfield.child)).
    ///
    /// Returns `true` if new input was received, `false` when nothing
    /// was received.
    fn poll_cmdline_output(&mut self) -> bool {
        let mut changed = false;

        if let Some((reader, child)) = &mut self.child {
            let mut line = String::new();
            const FLUSH_BUFFER_LIMIT: usize = 1024;
            for _ in 0..FLUSH_BUFFER_LIMIT {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Some(list) = self.lists.last_mut() {
                            list.items.push(line.clone());
                            changed = true;
                        }
                    }
                    _ => break,
                }
            }

            if let Some(status) = child
                .try_wait()
                .expect("Error attempting to wait for child output")
            {
                match status.code() {
                    Some(code) => {
                        if let Some(list) = self.lists.last_mut() {
                            list.items.push(format!(
                                "-- Execution Finished with status code: {} --",
                                code
                            ));
                            changed = true;
                        }
                    }
                    None => {
                        if let Some(list) = self.lists.last_mut() {
                            list.items
                                .push("-- Execution Terminated by a signal --".to_string());
                            changed = true;
                        }
                    }
                }
                self.child = None
            }
        }

        changed
    }

    fn handle_key(
        &mut self,
        key_stroke: KeyStroke,
        cmdline_result: &Option<String>,
        global: &mut Global,
    ) {
        if !global.handle_key(key_stroke) {
            match key_stroke {
                KeyStroke {
                    key: KEY_RETURN,
                    alt,
                } => {
                    if let Some(cmdline) = cmdline_result {
                        if alt {
                            self.run_cmdline(cmdline.clone());
                        } else {
                            // TODO(#47): endwin() on Enter in LineList looks like a total hack and it's unclear why it even works
                            endwin();
                            // TODO(#40): shell is not customizable
                            //   Grep for @ref(#40)
                            // TODO(#50): cm doesn't say anything if the executed command has failed
                            Command::new("sh")
                                .stdin(
                                    File::open("/dev/tty").expect(
                                        "Could not open /dev/tty as stdin for child process",
                                    ),
                                )
                                .arg("-c")
                                .arg(cmdline)
                                .spawn()
                                .expect("Could not spawn child process")
                                .wait_with_output()
                                .expect("Error waiting for output of child process");
                        }
                    }
                }
                KeyStroke {
                    key: KEY_BACKSPACE, ..
                } => {
                    self.lists.pop();
                }
                KeyStroke { key: KEY_F5, .. } => {
                    self.run_user_provided_cmdline();
                }
                key_stroke => {
                    if let Some(list) = self.lists.last_mut() {
                        list.handle_key(key_stroke);
                    }
                }
            }
        }
    }
}

#[derive(PartialEq)]
enum StringListState {
    Navigate,
    Editing { new: bool, prev_cursor_y: usize },
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

    fn handle_key(&mut self, key_stroke: KeyStroke, global: &mut Global) {
        match self.state {
            StringListState::Navigate => {
                if !global.handle_key(key_stroke) {
                    match key_stroke {
                        KeyStroke {
                            key: KEY_I,
                            alt: true,
                        } => {
                            self.list.duplicate_after();
                        }
                        KeyStroke {
                            key: KEY_SHIFT_I,
                            alt: true,
                        } => {
                            self.list.duplicate_before();
                        }
                        KeyStroke {
                            key: KEY_I,
                            alt: false,
                        } => {
                            self.state = StringListState::Editing {
                                new: true,
                                prev_cursor_y: self.list.cursor_y,
                            };
                            self.list.insert_after_current(String::new());
                            self.edit_field.buffer.clear();
                            self.edit_field.cursor_x = 0;
                            global.cursor_visible = true;
                        }
                        KeyStroke {
                            key: KEY_SHIFT_I,
                            alt: false,
                        } => {
                            self.state = StringListState::Editing {
                                new: true,
                                prev_cursor_y: self.list.cursor_y,
                            };
                            self.list.insert_before_current(String::new());
                            self.edit_field.buffer.clear();
                            self.edit_field.cursor_x = 0;
                            global.cursor_visible = true;
                        }
                        KeyStroke { key: KEY_F2, .. } => {
                            if let Some(item) = self.list.current_item() {
                                self.edit_field.cursor_x = item.len();
                                self.edit_field.buffer = String::from(item);
                                self.state = StringListState::Editing {
                                    new: false,
                                    prev_cursor_y: self.list.cursor_y,
                                };
                                global.cursor_visible = true;
                            }
                        }
                        key_stroke => self.list.handle_key(key_stroke),
                    }
                }
            }
            StringListState::Editing { new, prev_cursor_y } => match key_stroke {
                KeyStroke {
                    key: KEY_RETURN, ..
                } => {
                    self.state = StringListState::Navigate;
                    self.list.items[self.list.cursor_y] = self.edit_field.buffer.clone();
                    global.cursor_visible = false;
                }
                KeyStroke {
                    key: KEY_ESCAPE, ..
                } => {
                    self.state = StringListState::Navigate;
                    if new {
                        self.list.delete_current();
                        self.list.cursor_y = prev_cursor_y
                    }
                    global.cursor_visible = false;
                }
                key_stroke => self.edit_field.handle_key(key_stroke),
            },
        }
    }
}

fn render_status(y: usize, text: &str) {
    attron(COLOR_PAIR(REGULAR_PAIR));
    mv(y as i32, 0);
    addstr(text);
    attroff(COLOR_PAIR(REGULAR_PAIR));
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

    fn from_file(file_path: &Path) -> Self {
        let mut result = Profile::new();
        let input = read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Could not read file {}", file_path.display()));
        let (mut regex_count, mut cmd_count) = (0, 0);
        for (i, line) in input.lines().map(|x| x.trim_start()).enumerate() {
            // TODO(#128): profile parsing errors should be application error messages instead of Rust panics
            let fail = |message| panic!("{}:{}: {}", file_path.display(), i + 1, message);

            if !line.is_empty() {
                let mut assign = line.split('=');
                let key = assign
                    .next()
                    .unwrap_or_else(|| fail("Key is not provided"))
                    .trim();
                let value = assign
                    .next()
                    .unwrap_or_else(|| fail("Value is not provided"))
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
                            value.parse::<usize>().unwrap_or_else(|_| {
                                fail("Not a number");
                                0
                            })
                    }
                    "current_cmd" => {
                        result.cmd_list.list.cursor_y =
                            value.parse::<usize>().unwrap_or_else(|_| {
                                fail("Not a number");
                                0
                            })
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

        result
    }

    fn to_file<F: Write>(&self, stream: &mut F) {
        let error_message = "Could not save configuration";

        for regex in self.regex_list.list.items.iter() {
            writeln!(stream, "regexs = {}", regex).expect(error_message);
        }

        for cmd in self.cmd_list.list.items.iter() {
            writeln!(stream, "cmds = {}", cmd).expect(error_message);
        }

        writeln!(stream, "current_regex = {}", self.regex_list.list.cursor_y).expect(error_message);
        writeln!(stream, "current_cmd = {}", self.cmd_list.list.cursor_y).expect(error_message);
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
}

struct Global {
    /// Indicates that the Profile Panel, that contains Regex and Cmd
    /// lists is visible
    profile_pane: bool,
    /// Indicates that the application should quit the main event loop
    /// as soon as possible
    quit: bool,
    focus: Focus,
    cursor_visible: bool,
    cursor_x: i32,
    cursor_y: i32,
}

impl Global {
    fn handle_key(&mut self, key_stroke: KeyStroke) -> bool {
        match key_stroke {
            KeyStroke { key: KEY_E, .. } => {
                self.profile_pane = !self.profile_pane;
                true
            }
            KeyStroke { key: KEY_Q, .. } => {
                self.quit = true;
                true
            }
            KeyStroke { key: KEY_TAB, .. } => {
                self.focus = self.focus.next();
                true
            }
            _ => false,
        }
    }
}

fn render_cmdline(line: &str, cmd: &str, regex: &Regex) -> Option<String> {
    regex.captures_iter(line.as_bytes()).next().map(|cap_mat| {
        let mut result = cmd.to_string();
        if let Ok(caps) = cap_mat {
            for i in 1..caps.len() {
                if let Some(mat) = caps.get(i) {
                    result = result.replace(
                        format!("\\{}", i).as_str(),
                        line.get(mat.start()..mat.end()).unwrap_or(""),
                    )
                }
            }
        }
        result
    })
}

struct CmdlineEditField {
    edit_field: EditField,
    active: bool,
}

impl CmdlineEditField {
    fn new() -> Self {
        Self {
            edit_field: EditField::new(),
            active: false,
        }
    }

    fn activate(&mut self, line_list: &LineList, global: &mut Global) {
        self.active = true;

        if let Some(cmdline) = line_list.user_provided_cmdline.as_ref() {
            self.edit_field.buffer = cmdline.clone();
        } else {
            self.edit_field.buffer.clear();
        }

        self.edit_field.cursor_x = self.edit_field.buffer.len();
        global.cursor_visible = true;
    }

    fn render(&self, row: Row, global: &mut Global) {
        if self.active {
            self.edit_field.render(row);
            global.cursor_x = self.edit_field.cursor_x as i32;
            global.cursor_y = row.y as i32;
        }
    }

    fn handle_key(&mut self, key: KeyStroke, line_list: &mut LineList, global: &mut Global) {
        if self.active {
            match key {
                KeyStroke {key: KEY_RETURN, ..} => {
                    self.active = false;
                    global.cursor_visible = false;
                    line_list.user_provided_cmdline = Some(self.edit_field.buffer.clone());
                    line_list.run_user_provided_cmdline();
                },
                KeyStroke {key: KEY_ESCAPE, ..} => {
                    self.active = false;
                    global.cursor_visible = false;
                },
                _ => {
                    self.edit_field.handle_key(key)
                }
            }
        }
    }
}

fn main() {
    let config_path = {
        const CONFIG_FILE_NAME: &str = "cm.conf";
        let xdg_config_dir = var("XDG_CONFIG_HOME").map(PathBuf::from);
        let home_config_dir = var("HOME").map(PathBuf::from).map(|x| x.join(".config"));
        xdg_config_dir
            .or(home_config_dir)
            .map(|p| p.join(CONFIG_FILE_NAME))
            .expect("Could not find path to configuration file")
    };

    let mut profile = if config_path.exists() {
        Profile::from_file(&config_path)
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

    let mut cmdline_edit_field = CmdlineEditField::new();

    let mut line_list = LineList::new(std::env::args().nth(1));

    if line_list.user_provided_cmdline.is_some() {
        line_list.run_user_provided_cmdline();
    } else {
        let mut new_list = ItemList::new();
        new_list.items = stdin()
            .lock()
            .lines()
            .collect::<Result<Vec<String>, _>>()
            .expect("Error reading stdin");
        line_list.lists.push(new_list);
    }

    // NOTE: stolen from https://stackoverflow.com/a/44884859
    // TODO(#3): the terminal redirection is too hacky
    let tty_path = CString::new("/dev/tty").expect("Error trying to redirect stdin");
    let fopen_mode = CString::new("r+").expect("Error trying to redirect stdin");
    let file = unsafe { fopen(tty_path.as_ptr(), fopen_mode.as_ptr()) };
    let screen = newterm(None, file, file);
    set_term(screen);

    noecho();
    keypad(stdscr(), true);
    // NOTE: timeout(0) is a very important setting of ncurses for our
    // application. It makes getch() asynchronous, which is essential
    // for non-blocking UI when receiving the output from the child
    // process.
    timeout(0);

    init_style();

    let mut key_escaper = KeyEscaper::new();
    while !global.quit {
        // BEGIN INPUT SECTION //////////////////////////////
        // TODO(#43): cm does not handle Shift+TAB to scroll backwards through the panels
        let mut input_receved = false;
        let key = getch();
        if key != -1 {
            let cmdline = match (
                &profile.current_regex(),
                &profile.current_cmd(),
                &line_list.current_item(),
            ) {
                (Some(Ok(regex)), Some(cmd), Some(line)) => render_cmdline(line, &cmd, regex),
                _ => None,
            };

            if let Some(key_stroke) = key_escaper.feed(key) {
                input_receved = true;
                if cmdline_edit_field.active {
                    cmdline_edit_field.handle_key(key_stroke, &mut line_list, &mut global);
                } else {
                    match key_stroke {
                        KeyStroke {key: KEY_F3, ..} => {
                            cmdline_edit_field.activate(&line_list, &mut global);
                        }
                        _ => {
                            if !global.profile_pane {
                                line_list.handle_key(key_stroke, &cmdline, &mut global);
                            } else {
                                match global.focus {
                                    Focus::Lines => line_list.handle_key(key_stroke, &cmdline, &mut global),
                                    Focus::Regexs => profile.regex_list.handle_key(key_stroke, &mut global),
                                    Focus::Cmds => profile.cmd_list.handle_key(key_stroke, &mut global),
                                }
                            }
                        }
                    }
                }
            }
        }
        // END INPUT SECTION //////////////////////////////

        // BEGIN ASYNC CHILD OUTPUT SECTION //////////////////////////////
        let line_list_changed = line_list.poll_cmdline_output();
        // END ASYNC CHILD OUTPUT SECTION //////////////////////////////

        // BEGIN RENDER SECTION //////////////////////////////
        // NOTE: Don't try to rerender anything unless user provided some
        // input or the child process provided some output
        // TODO(#129): LineList::poll_cmdline_output() == true does not guarantee it is necessary to rerender
        //   If the output is appended outside of the screen it's kinda pointless to rerender
        if input_receved || line_list_changed {
            let (w, h) = {
                let mut x: i32 = 0;
                let mut y: i32 = 0;
                getmaxyx(stdscr(), &mut y, &mut x);
                (x as usize, y as usize)
            };

            erase();

            if h >= 1 {
                // NOTE: we are rerendering cmdline here because it could be changed by LineList
                // after the input handling section
                match (
                    &profile.current_regex(),
                    &profile.current_cmd(),
                    &line_list.current_item(),
                ) {
                    (Some(Ok(regex)), Some(cmd), Some(line)) => {
                        if let Some(cmdline) = render_cmdline(line, &cmd, regex) {
                            render_status(h - 1, &cmdline);
                        }
                    }
                    (Some(Err(err)), _, _) => render_status(h - 1, &err.to_string()),
                    _ => {}
                }
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

            cmdline_edit_field.render(Row{x: 0, y: h - 1, w}, &mut global);

            curs_set(if global.cursor_visible {
                ncurses::CURSOR_VISIBILITY::CURSOR_VISIBLE
            } else {
                ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE
            });
            mv(global.cursor_y, global.cursor_x);

            refresh();
        }
        // END RENDER SECTION //////////////////////////////

        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // TODO(#21): if application crashes it does not finalize the terminal
    endwin();

    config_path.parent().map(create_dir_all);
    profile.to_file(&mut File::create(config_path).expect("Could not open configuration file"));
}
