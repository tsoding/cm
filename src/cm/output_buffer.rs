use super::*;
use libc::*;
use ncurses::*;
use os_pipe::{pipe, PipeReader};
use pcre2::bytes::{Match, Regex};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::io::AsRawFd;
use std::process::{Child, Command};

// TODO(#94): mark_nonblocking does not work on Windows
fn mark_nonblocking<Fd: AsRawFd>(fd: &mut Fd) {
    unsafe {
        let flags = libc::fcntl(fd.as_raw_fd(), F_GETFL, 0);
        libc::fcntl(fd.as_raw_fd(), F_SETFL, flags | O_NONBLOCK);
    }
}

struct CharMatch {
    start: usize,
    end: usize,
}

fn byte_match_to_char_match(mat: &Match, s: &str) -> Option<CharMatch> {
    Some(CharMatch {
        start: s.get(0..mat.start())?.chars().count(),
        end: s.get(0..mat.end())?.chars().count(),
    })
}

struct ByteMatch {
    start: usize,
    end: usize,
}

fn char_match_to_byte_match(mat: ByteMatch, s: &str) -> ByteMatch {
    ByteMatch {
        start: s.chars().take(mat.start).collect::<String>().len(),
        end: s.chars().take(mat.end).collect::<String>().len(),
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

pub struct StatusLine {
    text_: String,
    error: bool,
}

impl StatusLine {
    pub fn new() -> Self {
        Self {
            text_: Default::default(),
            error: false,
        }
    }

    pub fn render(&self, y: usize) {
        let pair = if self.error { STATUS_ERROR_PAIR } else { REGULAR_PAIR };

        attron(COLOR_PAIR(pair));
        mv(y as i32, 0);
        addstr(&self.text_);
        attroff(COLOR_PAIR(pair));
    }

    pub fn set_text(&mut self, text: String) {
        self.text_ = text;
        self.error = false;
    }

    pub fn set_error(&mut self, text: String) {
        self.text_ = text;
        self.error = true;
    }

    pub fn clear(&mut self) {
        self.text_.clear();
        self.error = false;
    }
}

pub struct OutputBuffer {
    pub lists: Vec<ItemList<String>>,
    /// currently running process that generates data for OutputBuffer.
    /// See [OutputBuffer::poll_cmdline_output](struct.OutputBuffer.html#method.poll_cmdline_output)
    pub child: Option<(BufReader<PipeReader>, Child)>,
    pub status_line: StatusLine
}

impl OutputBuffer {
    pub fn new() -> Self {
        Self {
            lists: Vec::new(),
            child: None,
            status_line: StatusLine::new(),
        }
    }

    pub fn push(&mut self, line: String) {
        if let Some(list) = self.lists.last_mut() {
            list.items.push(line);
        }
    }

    pub fn current_item(&self) -> Option<&String> {
        self.lists.last().and_then(|x| x.current_item())
    }

    pub fn jump_to_next_match(&mut self, regex: &Regex) {
        if let Some(list) = self.lists.last_mut() {
            list.down();
            while !list.is_current_line_matches(regex) && !list.is_at_end() {
                list.down();
            }
        }
    }

    pub fn jump_to_prev_match(&mut self, regex: &Regex) {
        if let Some(list) = self.lists.last_mut() {
            list.up();
            while !list.is_current_line_matches(regex) && !list.is_at_begin() {
                list.up();
            }
        }
    }

    pub fn ctrlc(&mut self) {
        if cfg!(unix) {
            if let Some((_, child)) = &self.child {
                unsafe {
                    libc::kill(child.id() as i32, libc::SIGINT);
                }
            }
        }
    }

    pub fn render(
        &mut self,
        rect: Rect,
        focused: bool,
        regex_result: Option<Result<Regex, pcre2::Error>>,
    ) {
        if let Some(list) = self.lists.last_mut() {
            list.render(rect, focused);

            let Rect { x, y, w, h } = rect;
            if h > 0 {
                // TODO(#16): word wrapping for long lines
                for i in 0..h {
                    if list.scroll_y + i < list.items.len() {
                        let item = &list.items[list.scroll_y + i];
                        let selected = list.scroll_y + i == list.cursor_y;
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
                            // TODO(#189): regex capture highlighting is rendered with an offset
                            //   Probably due to pcre2 returning matches in bytes instead of chars
                            let cap_mats = regex.captures_iter(item.as_bytes()).next();
                            if let Some(cap_mat) = cap_mats {
                                if let Ok(caps) = cap_mat {
                                    // NOTE: we are skiping first cap because it contains the
                                    // whole match which is not needed in our case
                                    // TODO(#196): match highlighting does not respect the column width of the unicode characters
                                    for j in 1..caps.len() {
                                        if let Some(byte_mat) = caps.get(j) {
                                            // TODO(#197): test cm on incorrect utf-8 data
                                            let char_mat =
                                                byte_match_to_char_match(&byte_mat, item).unwrap();
                                            let char_start =
                                                usize::max(list.scroll_x, char_mat.start);
                                            let char_end =
                                                usize::min(list.scroll_x + w, char_mat.end);
                                            if char_start != char_end {
                                                let effective_byte_mat = char_match_to_byte_match(
                                                    ByteMatch {
                                                        start: char_start,
                                                        end: char_end,
                                                    },
                                                    item,
                                                );
                                                mv(
                                                    (y + i) as i32,
                                                    (char_start - list.scroll_x + x) as i32,
                                                );
                                                attron(COLOR_PAIR(cap_pair));
                                                addstr(
                                                    item.get(
                                                        effective_byte_mat.start
                                                            ..effective_byte_mat.end,
                                                    )
                                                    .unwrap_or(""),
                                                );
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
    }

    pub fn run_cmdline(&mut self, cmdline: String, shell: String) {
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
        let mut command = Command::new(shell);
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

    pub fn fork_cmdline(&mut self, cmdline: String, shell: String) {
        // NOTE: It is important to call endwin() function before running any child processes.
        // According to https://invisible-island.net/ncurses/man/curs_initscr.3x.html#h3-endwin
        //
        // > <..>
        // > A program should always call endwin before exiting or escaping from curses mode temporarily.
        // > <..>
        // > Calling refresh(3x) or doupdate(3x) after a temporary escape causes the program to resume visual mode.
        //
        // Add we will call refresh(3x) after the child process exists on the next iteration of the event loop
        endwin();
        // TODO(#40): shell is not customizable
        //   Grep for @ref(#40)

        let exit = Command::new(shell)
            .stdin(
                File::open("/dev/tty").expect("Could not open /dev/tty as stdin for child process"),
            )
            .arg("-c")
            .arg(cmdline.clone())
            .spawn()
            .expect("Could not spawn child process")
            .wait()
            .expect("Error waiting for output of child process");

        if !exit.success() {
            match exit.code() {
                Some(code) => self.status_line.set_error(format!("`{}` exited with code: {}", cmdline, code)),
                None => self.status_line.set_error(format!("`{}` was terminated by a signal", cmdline)),
            }
        }
    }

    /// Polls changes from the currently running child (see
    /// [OutputBuffer::run_cmdline](struct.OutputBuffer.html#method.run_cmdline),
    /// [OutputBuffer::child](struct.OutputBuffer.html#structfield.child)).
    ///
    /// Returns `true` if new input was received, `false` when nothing
    /// was received.
    pub fn poll_cmdline_output(&mut self) -> bool {
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
                            // TODO(#185): move the tab expansion to ItemList so it's available for every list-like component
                            list.items.push(expand_tabs(&line, TABSIZE() as usize));
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

    pub fn refresh_status_line(&mut self, profile: &Profile) {
        match (
            &profile.current_regex(),
            &profile.current_cmd(),
            &self.current_item(),
        ) {
            (Some(Ok(regex)), Some(cmd), Some(line)) => {
                let cmdline = render_cmdline(line, &cmd, regex).unwrap_or_default();
                self.status_line.set_text(cmdline);
            },
            (Some(Err(err)), _, _) => self.status_line.set_error(err.to_string()),
            _ => self.status_line.clear(),
        };
    }

    pub fn handle_key(
        &mut self,
        key_stroke: KeyStroke,
        profile: &Profile,
        global: &mut Global,
        shell: String,
    ) {
        let key_map = &profile.key_map;
        let regex_result = profile.current_regex();

        let cmdline_result = match (
            &regex_result,
            &profile.current_cmd(),
            &self.current_item(),
        ) {
            (Some(Ok(regex)), Some(cmd), Some(line)) => render_cmdline(line, &cmd, regex),
            _ => None,
        };

        if !global.handle_key(key_stroke, key_map) {
            if key_map.is_bound(key_stroke, action::RUN_INTO_ITSELF) {
                if let Some(cmdline) = &cmdline_result {
                    self.run_cmdline(cmdline.clone(), shell);
                    self.refresh_status_line(profile);
                }
            } else if key_map.is_bound(key_stroke, action::RUN) {
                if let Some(cmdline) = &cmdline_result {
                    self.fork_cmdline(cmdline.clone(), shell);
                }
            } else if key_map.is_bound(key_stroke, action::BACK) {
                self.lists.pop();
                self.refresh_status_line(profile);
            } else if key_map.is_bound(key_stroke, action::RERUN) {
                if let Some(cmdline) = global.user_provided_cmdline.clone() {
                    self.run_cmdline(cmdline, shell);
                    self.refresh_status_line(profile);
                }
            } else if key_map.is_bound(key_stroke, action::PREV_MATCH) {
                if let Some(Ok(regex)) = &regex_result {
                    self.jump_to_prev_match(&regex);
                    self.refresh_status_line(profile);
                }
            } else if key_map.is_bound(key_stroke, action::NEXT_MATCH) {
                if let Some(Ok(regex)) = &regex_result {
                    self.jump_to_next_match(&regex);
                    self.refresh_status_line(profile);
                }
            } else if key_map.is_bound(key_stroke, action::NEXT_SEARCH_MATCH) {
                if let Some(regex) = &global.search_regex {
                    self.jump_to_next_match(&regex);
                    self.refresh_status_line(profile);
                }
            } else if key_map.is_bound(key_stroke, action::PREV_SEARCH_MATCH) {
                if let Some(regex) = &global.search_regex {
                    self.jump_to_prev_match(&regex);
                    self.refresh_status_line(profile);
                }
            } else if let Some(list) = self.lists.last_mut() {
                list.handle_key(key_stroke, key_map);
                self.refresh_status_line(profile);
            }
        }
    }
}

/// Expands tab ('\t' 0x9) characters within an input string
/// into a variable amount of spaces.
///
/// ```text
/// |--------|    |--------|--------| 8 spaces/tab (tabsize = 8)
/// |\t      | => |........|        | 8 spaces
/// |\ta     | => |........|a       | 8 spaces + "a"
/// |aaa\t   | => |aaa.....|        | "aaa" + 5 spaces
/// ```
///
fn expand_tabs(input: &str, tabsize: usize) -> String {
    if tabsize == 0 {
        return input.replace('\t', "");
    }
    if tabsize == 1 {
        return input.replace('\t', " ");
    }

    let mut result =
        String::with_capacity(input.len() + (tabsize - 1) * input.matches('\t').count());
    let mut char_count = 0;

    for c in input.chars() {
        if c == '\t' {
            let space_count = tabsize - (char_count % tabsize);
            char_count += space_count;
            result.extend(std::iter::repeat(' ').take(space_count));
        } else {
            char_count += 1;
            result.push(c);
        }
    }

    result
}
