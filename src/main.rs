mod cm;

use ncurses::*;
use pcre2::bytes::Regex;
use std::env::var;
use std::fs::{create_dir_all, File};
use std::path::{PathBuf};
use cm::*;

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
