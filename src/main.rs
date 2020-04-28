use ncurses::*;
use regex::Regex;
use std::io::stdin;

const REGULAR_PAIR: i16 = 1;
const CURSOR_PAIR: i16 = 2;

fn render_list(lines: &Vec<String>, cursor: usize) {
    for (i, line) in lines.iter().enumerate() {
        mv(i as i32, 0);
        let pair = if i == cursor { CURSOR_PAIR } else { REGULAR_PAIR };
        attron(COLOR_PAIR(pair));
        printw(line.as_str());
        attroff(COLOR_PAIR(pair));
    }
}

fn main() -> Result<(), String> {
    let re = Regex::new(r"^(.*?):(\d+):").map_err(|e| e.to_string())?;

    // TODO: cm does not support input through pipes
    //   https://stackoverflow.com/a/44884859
    let mut lines: Vec<String> = Vec::new();
    let mut cursor: usize = 0;
    let mut line: String = String::new();
    while stdin().read_line(&mut line).map_err(|e| e.to_string())? > 0 {
        lines.push(line.clone());
        line.clear();
    }

    initscr();
    start_color();
    cbreak();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(CURSOR_PAIR, COLOR_BLACK, COLOR_WHITE);

    loop {
        erase();
        render_list(&lines, cursor);
        refresh();
        match getch() {
            115 => if cursor + 1 < lines.len() { cursor += 1 },
            119 => if cursor > 0 { cursor -= 1 },
            10 => {
                endwin();
                for cap in re.captures_iter(lines[cursor].as_str()) {
                    println!("vim +{} {}", &cap[2], &cap[1]);
                }
                return Ok(());
            }
            _ => {},
        }
    }

    endwin();

    Ok(())
}
