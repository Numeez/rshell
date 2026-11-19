#[derive(Debug, PartialEq)]
pub enum Command {
    Exit,
    Cd(Option<String>),
    Help,
    Invalid,
}
#[derive(Debug, PartialEq)]
pub enum Status {
    Stop,
    Continue,
}

pub fn parse_command(input: Vec<&str>) -> Option<Command> {
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
                return Some(Command::Cd(Some(val.to_string())));
            } else {
                Some(Command::Cd(None))
            }
        }
        "help" => Some(Command::Help),
        _ => Some(Command::Invalid),
    }
}

pub fn execute(cmd: Command) -> Status {
    match cmd {
        Command::Exit => Status::Stop,
        Command::Invalid => {
            eprintln!("Invalid command");
            Status::Continue
        }
        Command::Cd(target) => execute_cd(target),
        Command::Help => execute_help(),
    }
}
fn execute_cd(target: Option<String>) -> Status {
    if let Some(val) = target {
        if let Err(err) = std::env::set_current_dir(val) {
            eprintln!("Error occured while cd: {}", err);
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

#[cfg(test)]
mod tests {
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
    fn other_commands_are_not_exit() {
        assert_eq!(parse_command(vec!["ls", "-la"]), Some(Command::Invalid));
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
            Command::Cd(Some("/tmp".to_string())),
            parse_command(vec!["cd", "/tmp"]).unwrap()
        )
    }
    #[test]
    fn test_cd_command_with_more_than_one_args() {
        assert_eq!(
            Command::Cd(Some("/tmp".to_string())),
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
}
