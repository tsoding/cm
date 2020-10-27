use super::*;
use std::fs::{copy, read_to_string};
use std::path::Path;

pub type Type = fn(Vec<String>) -> Vec<String>;

fn migrate_v0_to_v1(mut lines: Vec<String>) -> Vec<String> {
    lines.push("shells = /bin/sh".to_string());
    lines.push("current_shell = 0".to_string());
    lines
}

fn migrate_v1_to_v2(mut lines: Vec<String>) -> Vec<String> {
    lines.push("key:PPAGE = page_up".to_string());
    lines.push("key:NPAGE = page_down".to_string());
    lines
}

fn migrate_v2_to_v3(lines: Vec<String>) -> Vec<String> {
    let mut new_lines = Vec::new();
    let mut shells = Vec::new();
    let mut current_shell = None;

    for line in lines.iter().map(|x| x.trim_start()) {
        if !line.is_empty() {
            let (key, value) = config::split_key_value(line)
                .unwrap_or_else(|| panic!("Invalid configuration line: {}", line));

            match key {
                "shells" => shells.push(value.to_string()),
                "current_shell" => {
                    current_shell = Some(
                        value
                            .parse::<usize>()
                            .unwrap_or_else(|_| panic!("Not a number: {}", value)),
                    )
                }
                _ => new_lines.push(line.to_string()),
            }
        }
    }

    new_lines.push(format!(
        "shell = {}",
        shells[current_shell
            .unwrap_or_else(|| panic!("Current shell was not provided by the config"))]
    ));

    new_lines
}

pub const CURRENT_VERSION: usize = 3;
pub const MIGRATIONS: [Type; CURRENT_VERSION] =
    [migrate_v0_to_v1, migrate_v1_to_v2, migrate_v2_to_v3];

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
