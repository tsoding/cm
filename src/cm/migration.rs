use std::fs::{read_to_string, copy};
use std::path::Path;

pub type Type = fn(Vec<String>) -> Vec<String>;

fn migrate_v0_to_v1(mut lines: Vec<String>) -> Vec<String> {
    lines.push("shells = /bin/sh".to_string());
    lines.push("current_shell = 0".to_string());
    lines
}

pub const CURRENT_VERSION: usize = 1;
pub const MIGRATIONS: [Type; CURRENT_VERSION] = [migrate_v0_to_v1];

pub fn read_and_migrate_file(filepath: &Path) -> Vec<String> {
    let input = read_to_string(filepath).unwrap();
    let mut lines = input.lines();
    if let Some(version_line) = lines.next() {
        match version_line
            .split('=')
            .map(|x| x.trim())
            .collect::<Vec<&str>>()
            .as_slice()
        {
            ["version", number] => {
                let mut version_number = number.parse::<usize>().unwrap();
                let mut input = lines.map(|x| x.to_string()).collect::<Vec<String>>();

                if version_number < CURRENT_VERSION {
                    copy(filepath, format!("{}.bak", filepath.display())).unwrap();
                }

                while version_number < CURRENT_VERSION {
                    input = MIGRATIONS[version_number](input);
                    version_number += 1;
                }
                return input;
            }
            _ => {
                panic!("Version line is not correct");
            }
        }
    }

    panic!("Version line is not found");
}
