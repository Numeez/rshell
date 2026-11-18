#[derive(Debug, PartialEq)]
pub enum Command {
    Exit,
    Invalid,
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
        _ => Some(Command::Invalid),
    }
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
}
