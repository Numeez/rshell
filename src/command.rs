use std::{
    ffi::{CString, NulError, c_char},
    ptr,
};

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Exit,
    Cd(Option<&'a str>),
    Help,
    External(Vec<&'a str>),
}
#[derive(Debug, PartialEq)]
pub enum Status {
    Stop,
    Continue,
}

pub fn parse_command(input: Vec<&str>) -> Option<Command<'_>> {
    if input.is_empty() {
        return None;
    }
    let command = input[0];
    match command.to_ascii_lowercase().as_str() {
        "exit" => {
            return Some(Command::Exit);
        }
        "cd" => {
            let target = input.get(1);
            if let Some(val) = target {
                return Some(Command::Cd(Some(val)));
            } else {
                Some(Command::Cd(None))
            }
        }
        "help" => Some(Command::Help),
        _ => Some(Command::External(input)),
    }
}

pub fn execute(cmd: Command) -> Status {
    match cmd {
        Command::Exit => Status::Stop,
        Command::Cd(target) => execute_cd(target),
        Command::Help => execute_help(),
        Command::External(val) => execute_external_command(val),
    }
}
fn execute_cd(target: Option<&str>) -> Status {
    if let Some(val) = target {
        if let Err(err) = std::env::set_current_dir(val) {
            eprintln!("Error occured while cd: {}", err);
            return Status::Continue;
        }
        match std::env::current_dir() {
            Ok(val) => {
                println!("cd: {}", val.as_os_str().to_string_lossy());
            }
            Err(e) => {
                eprintln!("Error occured while cd: {}", e);
            }
        }
    } else {
        eprintln!("rsh: cd needs an argument");
    }
    return Status::Continue;
}
fn execute_help() -> Status {
    let help_message = concat!(
        "rsh: a simple shell written in Rust, based on Stephen Brennan's LSH\n",
        "Type a program name and its arguments, then press Enter.\n",
        "\n",
        "Built-in commands:\n",
        "  cd <dir>   change the current directory\n",
        "  help       show this message\n",
        "  exit       quit the shell\n",
        "\n",
        "Use the man command for information on other programs.",
    );
    println!("{}", help_message);
    Status::Continue
}

fn execute_external_command(cmd: Vec<&str>) -> Status {
    let c_strings = to_c_strings(&cmd);
    match c_strings {
        Ok(val) => {
            let c_pointers = to_argv(&val);
        }
        Err(e) => {
            eprintln!("rsh: {}", e);
            return Status::Continue;
        }
    }

    Status::Continue
}

fn to_c_strings(words: &[&str]) -> Result<Vec<CString>, NulError> {
    let mut c_strings = Vec::new();
    for word in words {
        c_strings.push(CString::new(*word)?);
    }
    Ok(c_strings)
}

fn to_argv(c_strings: &[CString]) -> Vec<*const c_char> {
    let mut argv = Vec::new();
    for c_string in c_strings {
        argv.push(c_string.as_ptr());
    }
    argv.push(std::ptr::null());
    argv
}
#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;

    #[test]
    fn exit_alone_exits() {
        assert_eq!(parse_command(vec!["exit"]), Some(Command::Exit));
    }

    #[test]
    fn exit_with_arguments_still_exits() {
        assert_eq!(parse_command(vec!["exit", "now"]), Some(Command::Exit));
    }

    #[test]
    fn empty_input_is_none() {
        assert_eq!(parse_command(vec![]), None);
    }
    #[test]
    fn test_cd_command() {
        assert_eq!(Command::Cd(None), parse_command(vec!["cd"]).unwrap())
    }
    #[test]
    fn test_cd_command_with_args() {
        assert_eq!(
            Command::Cd(Some("/tmp")),
            parse_command(vec!["cd", "/tmp"]).unwrap()
        )
    }
    #[test]
    fn test_cd_command_with_more_than_one_args() {
        assert_eq!(
            Command::Cd(Some("/tmp")),
            parse_command(vec!["cd", "/tmp", "var"]).unwrap()
        )
    }
    #[test]
    fn test_command_exit() {
        assert_eq!(Status::Stop, execute(Command::Exit))
    }
    #[test]
    fn test_help_command() {
        assert_eq!(Command::Help, parse_command(vec!["help"]).unwrap());
        assert_eq!(Command::Help, parse_command(vec!["help", "tmp"]).unwrap())
    }

    #[test]
    fn test_external_command() {
        assert_eq!(
            Command::External(vec!["ls"]),
            parse_command(vec!["ls"]).unwrap()
        );
        assert_eq!(
            Command::External(vec!["ls", "-la"]),
            parse_command(vec!["ls", "-la"]).unwrap()
        );
    }
    #[test]
    fn builds_null_terminated_argv() {
        let c_strings = to_c_strings(&["ls", "-la"]).unwrap();
        let argv = to_argv(&c_strings);

        assert_eq!(argv.len(), 3);
        assert!(argv[2].is_null());

        let first = unsafe { CStr::from_ptr(argv[0]) };
        let second = unsafe { CStr::from_ptr(argv[1]) };
        assert_eq!(first.to_str().unwrap(), "ls");
        assert_eq!(second.to_str().unwrap(), "-la");
    }

    #[test]
    fn rejects_word_with_nul_byte() {
        assert!(to_c_strings(&["echo", "a\0b"]).is_err());
    }
}
