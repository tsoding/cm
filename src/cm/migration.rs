use std::fs::read_to_string;
use std::path::Path;

pub type Type = fn(Vec<String>) -> Vec<String>;

pub const CURRENT_VERSION: usize = 0;
pub const MIGRATIONS: [Type; CURRENT_VERSION] = [];

// NOTE: One of the important features of read_and_migrate_file is
// that it never saves the migrated file. It is saved at the end of
// the life of the application (grep for @migrate-fail). Because of
// that if the migration fails, the original configuration file will
// not be corrupted.
//
// It is very important to not mess with that behaviour!

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
                #[allow(clippy::absurd_extreme_comparisons)]
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
