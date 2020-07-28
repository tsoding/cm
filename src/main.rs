mod cm;

use libc::*;
use ncurses::*;
use os_pipe::{pipe, PipeReader};
use pcre2::bytes::Regex;
use std::env::var;
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader};
use std::os::unix::io::AsRawFd;
use std::path::{PathBuf};
use std::process::{Child, Command};
use cm::*;

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

    fn jump_to_next_match(&mut self, regex: &Regex) {
        if let Some(list) = self.lists.last_mut() {
            list.down();
            while !list.is_current_line_matches(regex) && !list.is_at_end() {
                list.down();
            }
        }
    }

    fn jump_to_prev_match(&mut self, regex: &Regex) {
        if let Some(list) = self.lists.last_mut() {
            list.up();
            while !list.is_current_line_matches(regex) && !list.is_at_begin() {
                list.up();
            }
        }
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

    fn fork_cmdline(&mut self, cmdline: String) {
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
        regex_result: Option<Result<Regex, pcre2::Error>>,
        global: &mut Global,
    ) {
        if !global.handle_key(key_stroke) {
            match key_stroke {
                KeyStroke { key: KEY_RETURN, alt: true } => {
                    if let Some(cmdline) = cmdline_result {
                        self.run_cmdline(cmdline.clone());
                    }
                }
                KeyStroke {key: KEY_RETURN, alt: false} => {
                    if let Some(cmdline) = cmdline_result {
                        self.fork_cmdline(cmdline.clone());
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
                KeyStroke {
                    key: KEY_UP,
                    alt: true,
                } => {
                    if let Some(Ok(regex)) = regex_result {
                        self.jump_to_prev_match(&regex);
                    }
                }
                KeyStroke {
                    key: KEY_DOWN,
                    alt: true,
                } => {
                    if let Some(Ok(regex)) = regex_result {
                        self.jump_to_next_match(&regex);
                    }
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


fn render_status(y: usize, text: &str) {
    attron(COLOR_PAIR(REGULAR_PAIR));
    mv(y as i32, 0);
    addstr(text);
    attroff(COLOR_PAIR(REGULAR_PAIR));
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

    fn accept_editing(&mut self, line_list: &mut LineList, global: &mut Global) {
        self.active = false;
        global.cursor_visible = false;
        line_list.user_provided_cmdline = Some(self.edit_field.buffer.clone());
        line_list.run_user_provided_cmdline();
    }

    fn cancel_editing(&mut self, global: &mut Global) {
        self.active = false;
        global.cursor_visible = false;
    }

    fn handle_key(&mut self, key: KeyStroke, line_list: &mut LineList, global: &mut Global) {
        if self.active {
            match key {
                KeyStroke {
                    key: KEY_RETURN, ..
                } => {
                    self.accept_editing(line_list, global);
                }
                KeyStroke {
                    key: KEY_ESCAPE, ..
                } => {
                    self.cancel_editing(global);
                }
                _ => self.edit_field.handle_key(key),
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
    }

    initscr();
    // NOTE: timeout(16) is a very important setting of ncurses for our
    // application. It makes getch() asynchronous, which is essential
    // for non-blocking UI when receiving the output from the child
    // process.
    //
    // The value of 16 milliseconds also blocks the application for a
    // little. This improves the performance by making the application
    // to not constantly busy loop on checking the input from the user
    // and running child process.
    //
    // 16 milliseconds were chosen to make the application "run in 60 fps" :D
    timeout(16);
    noecho();
    keypad(stdscr(), true);

    init_style();

    while !global.quit {
        // BEGIN INPUT SECTION //////////////////////////////
        // TODO(#43): cm does not handle Shift+TAB to scroll backwards through the panels
        let mut input_receved = false;
        if let Some(key_stroke) = KeyStroke::get() {
            input_receved = true;

            let cmdline = match (
                &profile.current_regex(),
                &profile.current_cmd(),
                &line_list.current_item(),
            ) {
                (Some(Ok(regex)), Some(cmd), Some(line)) => render_cmdline(line, &cmd, regex),
                _ => None,
            };

            if cmdline_edit_field.active {
                cmdline_edit_field.handle_key(key_stroke, &mut line_list, &mut global);
            } else {
                match key_stroke {
                    KeyStroke { key: KEY_F3, .. } => {
                        cmdline_edit_field.activate(&line_list, &mut global);
                    }
                    _ => {
                        if !global.profile_pane {
                            line_list.handle_key(
                                key_stroke,
                                &cmdline,
                                profile.current_regex(),
                                &mut global,
                            );
                        } else {
                            match global.focus {
                                Focus::Lines => line_list.handle_key(
                                    key_stroke,
                                    &cmdline,
                                    profile.current_regex(),
                                    &mut global,
                                ),
                                Focus::Regexs => {
                                    profile.regex_list.handle_key(key_stroke, &mut global)
                                }
                                Focus::Cmds => profile.cmd_list.handle_key(key_stroke, &mut global),
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

            cmdline_edit_field.render(Row { x: 0, y: h - 1, w }, &mut global);

            curs_set(if global.cursor_visible {
                ncurses::CURSOR_VISIBILITY::CURSOR_VISIBLE
            } else {
                ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE
            });
            mv(global.cursor_y, global.cursor_x);

            refresh();
        }
        // END RENDER SECTION //////////////////////////////
    }

    // TODO(#21): if application crashes it does not finalize the terminal
    endwin();

    config_path.parent().map(create_dir_all);
    profile.to_file(&mut File::create(config_path).expect("Could not open configuration file"));
}
