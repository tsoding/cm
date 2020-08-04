use super::*;
use pcre2::bytes::Regex;
use std::fs::read_to_string;
use std::io::Write;
use std::path::Path;

pub struct Profile {
    pub regex_list: StringList,
    pub cmd_list: StringList,
    pub key_map: KeyMap,
}

impl Profile {
    pub fn new() -> Self {
        Self {
            regex_list: StringList::new(),
            cmd_list: StringList::new(),
            key_map: KeyMap::new(),
        }
    }

    pub fn from_file(file_path: &Path) -> Self {
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

    pub fn to_file<F: Write>(&self, stream: &mut F) {
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

    pub fn current_regex(&self) -> Option<Result<Regex, pcre2::Error>> {
        match self.regex_list.state {
            StringListState::Navigate => self.regex_list.current_item().map(|s| Regex::new(&s)),
            StringListState::Editing { .. } => Some(Regex::new(&self.regex_list.edit_field.buffer)),
        }
    }

    pub fn current_cmd(&self) -> Option<String> {
        match self.cmd_list.state {
            StringListState::Navigate => self.cmd_list.current_item().map(String::from),
            StringListState::Editing { .. } => Some(self.cmd_list.edit_field.buffer.clone()),
        }
    }

    pub fn initial() -> Self {
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
        result.key_map = KeyMap::initial();
        result
    }
}
