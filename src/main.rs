mod cm;

use cm::*;
use ncurses::*;
use pcre2::bytes::Regex;
use std::env::var;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;

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
    };

    let mut cursor = Cursor {
        x: 0,
        y: 0,
        visible: false,
    };

    let mut key_map = KeyMap::new();
    key_map.bind(KeyStroke {key: KEY_UP, alt: false}, Action::Up);
    key_map.bind(KeyStroke {key: KEY_DOWN, alt: false}, Action::Down);
    key_map.bind(KeyStroke {key: KEY_LEFT, alt: false}, Action::Left);
    key_map.bind(KeyStroke {key: KEY_RIGHT, alt: false}, Action::Right);
    key_map.bind(KeyStroke {key: KEY_HOME, alt: false}, Action::Home);
    key_map.bind(KeyStroke {key: KEY_DC, alt: false}, Action::DeleteItem);

    let mut cmdline_edit_field = CmdlineEditField::new();

    let mut output_buffer = OutputBuffer::new(std::env::args().nth(1));

    if output_buffer.user_provided_cmdline.is_some() {
        output_buffer.run_user_provided_cmdline();
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
        let mut input_receved = false;
        if let Some(key_stroke) = KeyStroke::get() {
            input_receved = true;

            let cmdline = match (
                &profile.current_regex(),
                &profile.current_cmd(),
                &output_buffer.current_item(),
            ) {
                (Some(Ok(regex)), Some(cmd), Some(line)) => render_cmdline(line, &cmd, regex),
                _ => None,
            };

            if cmdline_edit_field.active {
                cmdline_edit_field.handle_key(key_stroke, &mut output_buffer, &mut cursor);
            } else {
                match key_stroke {
                    KeyStroke { key: KEY_F3, .. } => {
                        cmdline_edit_field.activate(&output_buffer, &mut cursor);
                    }
                    _ => {
                        if !global.profile_pane {
                            output_buffer.handle_key(
                                key_stroke,
                                &key_map,
                                &cmdline,
                                profile.current_regex(),
                                &mut global,
                            );
                        } else {
                            match global.focus {
                                Focus::Lines => output_buffer.handle_key(
                                    key_stroke,
                                    &key_map,
                                    &cmdline,
                                    profile.current_regex(),
                                    &mut global,
                                ),
                                Focus::Regexs => profile.regex_list.handle_key(
                                    key_stroke,
                                    &key_map,
                                    &mut global,
                                    &mut cursor,
                                ),
                                Focus::Cmds => profile.cmd_list.handle_key(
                                    key_stroke,
                                    &key_map,
                                    &mut global,
                                    &mut cursor,
                                ),
                            }
                        }
                    }
                }
            }
        }

        // END INPUT SECTION //////////////////////////////

        // BEGIN ASYNC CHILD OUTPUT SECTION //////////////////////////////
        let output_buffer_changed = output_buffer.poll_cmdline_output();
        // END ASYNC CHILD OUTPUT SECTION //////////////////////////////

        // BEGIN RENDER SECTION //////////////////////////////
        // NOTE: Don't try to rerender anything unless user provided some
        // input or the child process provided some output
        // TODO(#129): OutputBuffer::poll_cmdline_output() == true does not guarantee it is necessary to rerender
        //   If the output is appended outside of the screen it's kinda pointless to rerender
        if input_receved || output_buffer_changed {
            let (w, h) = {
                let mut x: i32 = 0;
                let mut y: i32 = 0;
                getmaxyx(stdscr(), &mut y, &mut x);
                (x as usize, y as usize)
            };

            erase();

            if h >= 1 {
                // NOTE: we are rerendering cmdline here because it could be changed by OutputBuffer
                // after the input handling section
                match (
                    &profile.current_regex(),
                    &profile.current_cmd(),
                    &output_buffer.current_item(),
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

            let working_rect = Rect {
                x: 0,
                y: 0,
                w,
                h: h - 1,
            };
            if global.profile_pane {
                let (output_buffer_rect, profile_rect) = working_rect.horizontal_split(3);
                let (regex_rect, cmd_rect) = profile_rect.vertical_split(2);

                output_buffer.render(
                    output_buffer_rect,
                    global.focus == Focus::Lines,
                    profile.current_regex(),
                );
                profile
                    .regex_list
                    .render(regex_rect, global.focus == Focus::Regexs, &mut cursor);
                profile
                    .cmd_list
                    .render(cmd_rect, global.focus == Focus::Cmds, &mut cursor);
            } else {
                output_buffer.render(working_rect, true, profile.current_regex());
            }

            cmdline_edit_field.render(Row { x: 0, y: h - 1, w }, &mut cursor);

            cursor.sync();

            refresh();
        }
        // END RENDER SECTION //////////////////////////////
    }

    // TODO(#21): if application crashes it does not finalize the terminal
    endwin();

    config_path.parent().map(create_dir_all);
    profile.to_file(&mut File::create(config_path).expect("Could not open configuration file"));
}
