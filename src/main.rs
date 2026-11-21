mod command;
mod ffi;
mod wait;

use std::{
    ffi::{OsStr, c_int},
    io::{self, Write, stdout},
};

use crate::command::{Status, execute, parse_command};

fn main() {
    let mut status_code: c_int = 0;
    loop {
        let user = std::env::var("USER").unwrap_or_default();
        let current_dir = match std::env::current_dir() {
            Ok(val) => val
                .file_name()
                .unwrap_or(OsStr::new("/"))
                .to_string_lossy()
                .to_string(),
            Err(_) => String::from("?"),
        };
        let start_icon = format!(
            "\x1b[1;32m{}@rsh\x1b[0m \x1b[34m{}\x1b[0m $",
            user, current_dir
        );
        print!("{start_icon} ");
        let _ = stdout().flush();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    println!();
                    break;
                }
                let status_text = status_code.to_string();
                let mut values = split_words(&line);
                for word in values.iter_mut() {
                    if *word == "$?" {
                        *word = &status_text;
                    }
                }
                if let Some(cmd) = parse_command(values) {
                    match execute(cmd) {
                        Status::Stop => break,
                        Status::Continue(val) => {
                            status_code = val;
                        }
                    }
                }
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::InvalidData {
                    eprintln!("error: {}", err);
                } else {
                    eprintln!("error: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
    std::process::exit(status_code);
}

fn split_words(line: &str) -> Vec<&str> {
    let values: Vec<&str> = line.split_whitespace().collect();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_simple_command() {
        assert_eq!(split_words("ls -la /tmp"), vec!["ls", "-la", "/tmp"]);
    }

    #[test]
    fn collapses_extra_whitespace() {
        assert_eq!(split_words("   ls    -la  "), vec!["ls", "-la"]);
        assert_eq!(split_words("ls\t-la\t\t/tmp"), vec!["ls", "-la", "/tmp"]);
        assert_eq!(split_words(" \t ls \t -la"), vec!["ls", "-la"]);
    }

    #[test]
    fn strips_trailing_newline() {
        assert_eq!(split_words("ls -la\n"), vec!["ls", "-la"]);
        assert_eq!(split_words("echo hi\r\n"), vec!["echo", "hi"]);
    }

    #[test]
    fn empty_input_gives_no_words() {
        assert!(split_words("").is_empty());
        assert!(split_words("   ").is_empty());
        assert!(split_words("\n").is_empty());
        assert!(split_words(" \t \n").is_empty());
    }
}
