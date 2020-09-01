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
        Profile::from_file(migration::read_and_migrate_file(&config_path), &config_path)
    } else {
        Profile::initial()
    };

    let mut key_map_settings = KeyMapSettings::new();

    let mut global = Global::new(std::env::args().nth(1));

    let mut output_buffer = OutputBuffer::new();

    if global.user_provided_cmdline.is_none() {
        output_buffer.lists.push(ItemList::new());
        output_buffer.push("Welcome to cm!".to_string());
        output_buffer.push("- Use arrows or vim style hjkl to navigate.".to_string());
        output_buffer.push("- Press F3 to enter a command to run.".to_string());
        // TODO: tutorial does not respect current key bindings
    }

    if let Some(cmdline) = global.user_provided_cmdline.clone() {
        output_buffer.run_cmdline(cmdline);
    }

    initscr();
    // NOTE(timeout): timeout(16) is a very important setting of ncurses for our
    // application. It makes getch() asynchronous, which is essential
    // for non-blocking UI when receiving the output from the child
    // process.
    //
    // The value of 16 milliseconds also blocks the application for a
    // little. This improves the performance by making the application
    // to not constantly busy loop on checking the input from the user
    // and the running child process.
    //
    // 16 milliseconds were chosen to make the application "run in 60 fps" :D
    timeout(16);
    noecho();
    keypad(stdscr(), true);

    init_style();

    // NOTE(rerender): because of the asynchronous nature of the application the
    // rendering process could be invoked every 16 millisecond (See NOTE(timeout)),
    // which is expensive, so we introduce a simple boolean variable that is changed
    // through out a single iteration of the Event Loop in cases when the state of the
    // application is potentially changed which needs to be reflected by rerendering
    // the screen.
    //
    // Grep for NOTE(rerender) for more info.
    let mut rerender = true;
    while !global.quit {
        // BEGIN INPUT SECTION //////////////////////////////
        if let Some(key_stroke) = KeyStroke::get() {
            // NOTE(rerender): at the point the user provided some input which potentially
            // changes the state of the application which needs to be reflected by rerendering
            // the screen.
            rerender = true;

            let cmdline = match (
                &profile.current_regex(),
                &profile.current_cmd(),
                &output_buffer.current_item(),
            ) {
                (Some(Ok(regex)), Some(cmd), Some(line)) => render_cmdline(line, &cmd, regex),
                _ => None,
            };

            if global.key_map_settings {
                key_map_settings.handle_key(key_stroke, &mut profile.key_map, &mut global)
            } else if global.bottom_state != BottomState::Nothing {
                if profile.key_map.is_bound(key_stroke, action::ACCEPT) {
                    global.bottom_edit_field.stop_editing(&mut global.cursor);

                    match global.bottom_state {
                        BottomState::Cmdline => {
                            global.user_provided_cmdline =
                                Some(global.bottom_edit_field.edit_field.buffer.clone());
                            output_buffer
                                .run_cmdline(global.bottom_edit_field.edit_field.buffer.clone());
                        }
                        BottomState::Search => {
                            if let Ok(regex) =
                                Regex::new(global.bottom_edit_field.edit_field.buffer.as_str())
                            {
                                output_buffer.jump_to_next_match(&regex);
                                global.search_regex = Some(regex);
                            }
                        }
                        BottomState::Nothing => {
                            unreachable!("Unexpected bottom state");
                        }
                    }
                    global.bottom_state = BottomState::Nothing;
                } else if profile.key_map.is_bound(key_stroke, action::CANCEL) {
                    global.bottom_edit_field.stop_editing(&mut global.cursor);
                    global.bottom_state = BottomState::Nothing;
                } else {
                    global
                        .bottom_edit_field
                        .handle_key(key_stroke, &profile.key_map);
                }
            } else if !global.profile_pane {
                output_buffer.handle_key(
                    key_stroke,
                    &profile.key_map,
                    &cmdline,
                    profile.current_regex(),
                    &mut global,
                );
            } else {
                match global.focus {
                    Focus::Output => output_buffer.handle_key(
                        key_stroke,
                        &profile.key_map,
                        &cmdline,
                        profile.current_regex(),
                        &mut global,
                    ),
                    Focus::Regexs => {
                        profile
                            .regex_list
                            .handle_key(key_stroke, &profile.key_map, &mut global)
                    }
                    Focus::Cmds => {
                        profile
                            .cmd_list
                            .handle_key(key_stroke, &profile.key_map, &mut global)
                    }
                }
            }
        }
        // END INPUT SECTION //////////////////////////////

        // BEGIN ASYNC CHILD OUTPUT SECTION //////////////////////////////
        {
            // TODO(#129): OutputBuffer::poll_cmdline_output() == true does not guarantee it is necessary to rerender
            //   If the output is appended outside of the screen it's kinda pointless to rerender
            let output_buffer_changed = output_buffer.poll_cmdline_output();
            // NOTE(rerender): output_buffer_changed == true means we recieved some output
            // from the currently running child process and the output is pushed to the
            // output_buffer which effectevly changes the state of the application which needs
            // to be reflected by rerendering the screen.
            rerender = rerender || output_buffer_changed;
        }
        // END ASYNC CHILD OUTPUT SECTION //////////////////////////////

        // BEGIN RENDER SECTION //////////////////////////////
        // NOTE(rerender): Don't try to rerender anything unless the state of the application has
        // changed
        if rerender {
            let (w, h) = {
                let mut x: i32 = 0;
                let mut y: i32 = 0;
                getmaxyx(stdscr(), &mut y, &mut x);
                (x as usize, y as usize)
            };

            erase();

            if global.key_map_settings {
                key_map_settings.render(Rect { x: 0, y: 0, w, h }, true);
            } else {
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
                        global.focus == Focus::Output,
                        profile.current_regex(),
                    );
                    profile.regex_list.render(
                        regex_rect,
                        global.focus == Focus::Regexs,
                        &mut global.cursor,
                    );
                    profile.cmd_list.render(
                        cmd_rect,
                        global.focus == Focus::Cmds,
                        &mut global.cursor,
                    );
                } else {
                    output_buffer.render(working_rect, true, profile.current_regex());
                }

                if global.bottom_state != BottomState::Nothing {
                    global
                        .bottom_edit_field
                        .render(Row { x: 0, y: h - 1, w }, &mut global.cursor);
                }
            }

            global.cursor.sync();

            refresh();
        }
        // END RENDER SECTION //////////////////////////////

        // NOTE(rerender): Don't assume rerendering on the next iteration of the Event Loop.
        rerender = false;
    }

    // TODO(#21): if application crashes it does not finalize the terminal
    endwin();

    config_path.parent().map(create_dir_all);
    profile
        .to_file(&mut File::create(config_path).expect("Could not open configuration file"))
        .expect("Could not save configuration");
}
