use std::{
    ffi::{CString, NulError, c_char, c_int},
    io::Error,
};

use crate::{
    ffi::{_exit, close, dup2, execvp, fork, pid_t, pipe, waitpid},
    wait::{WaitStatus, get_status},
};

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Exit,
    Cd(Option<&'a str>),
    Help,
    Pipeline(Vec<Vec<&'a str>>),
    SyntaxError,
}
#[derive(Debug, PartialEq)]
pub enum Status {
    Stop,
    Continue(c_int),
}

pub fn parse_command(input: Vec<&str>) -> Option<Command<'_>> {
    if input.is_empty() {
        return None;
    }
    if input.contains(&"|") {
        let stages: Vec<Vec<&str>> = input
            .split(|word| *word == "|")
            .map(|stage| stage.to_vec())
            .collect();

        if stages.iter().any(|stage| stage.is_empty()) {
            return Some(Command::SyntaxError);
        }
        return Some(Command::Pipeline(stages));
    }

    match input[0].to_ascii_lowercase().as_str() {
        "exit" => Some(Command::Exit),
        "cd" => Some(Command::Cd(input.get(1).copied())),
        "help" => Some(Command::Help),
        _ => Some(Command::Pipeline(vec![input])),
    }
}

pub fn execute(cmd: Command) -> Status {
    match cmd {
        Command::Exit => Status::Stop,
        Command::Cd(target) => execute_cd(target),
        Command::Help => execute_help(),
        Command::Pipeline(stages) => execute_pipeline(stages),
        Command::SyntaxError => {
            eprintln!("error: syntax error near `|`");
            Status::Continue(2)
        }
    }
}
fn execute_cd(target: Option<&str>) -> Status {
    let mut status: c_int = 0;
    if let Some(val) = target {
        if let Err(err) = std::env::set_current_dir(val) {
            eprintln!("error: cd: {}", err);
            return Status::Continue(1);
        }
    } else {
        status = 1;
        eprintln!("error: cd: needs an argument");
    }
    Status::Continue(status)
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
    Status::Continue(0)
}

fn execute_pipeline(stages: Vec<Vec<&str>>) -> Status {
    let mut c_strings = Vec::new();
    for stage in &stages {
        match to_c_strings(stage) {
            Ok(strings) => c_strings.push(strings),
            Err(e) => {
                eprintln!("error: {}", e);
                return Status::Continue(1);
            }
        }
    }
    let argvs: Vec<Vec<*const c_char>> = c_strings.iter().map(|s| to_argv(s)).collect();

    let mut pids: Vec<pid_t> = Vec::new();
    let mut prev_read: Option<c_int> = None;

    for (i, argv) in argvs.iter().enumerate() {
        let is_last = i == argvs.len() - 1;

        // Every stage except the last writes into a new pipe.
        let (next_read, write) = if is_last {
            (None, None)
        } else {
            let mut fds: [c_int; 2] = [0, 0];
            if unsafe { pipe(fds.as_mut_ptr()) } == -1 {
                eprintln!("error: pipe: {}", Error::last_os_error());
                close_fd(prev_read);
                wait_all(&pids);
                return Status::Continue(1);
            }
            (Some(fds[0]), Some(fds[1]))
        };

        let pid = spawn(argv, stages[i][0], prev_read, write, next_read);
        close_fd(prev_read);
        close_fd(write);

        match pid {
            Some(pid) => pids.push(pid),
            None => {
                close_fd(next_read);
                wait_all(&pids);
                return Status::Continue(1);
            }
        }

        prev_read = next_read;
    }

    let mut last_status = 0;
    for pid in &pids {
        last_status = wait_for(*pid);
    }
    Status::Continue(last_status)
}

fn spawn(
    argv: &[*const c_char],
    name: &str,
    stdin_fd: Option<c_int>,
    stdout_fd: Option<c_int>,
    unused_fd: Option<c_int>,
) -> Option<pid_t> {
    let pid = unsafe { fork() };
    if pid == -1 {
        eprintln!("error: fork: {}", Error::last_os_error());
        return None;
    }
    if pid == 0 {
        unsafe {
            if let Some(fd) = stdin_fd {
                dup2(fd, 0);
                close(fd);
            }
            if let Some(fd) = stdout_fd {
                dup2(fd, 1);
                close(fd);
            }
            if let Some(fd) = unused_fd {
                close(fd);
            }
            execvp(argv[0], argv.as_ptr());
            eprintln!("error: {}: {}", name, Error::last_os_error());
            _exit(1);
        }
    }
    Some(pid)
}

fn wait_for(pid: pid_t) -> c_int {
    let mut raw: c_int = 0;
    if unsafe { waitpid(pid, &mut raw, 0) } == -1 {
        eprintln!("error: waitpid: {}", Error::last_os_error());
        return 1;
    }
    match get_status(raw) {
        WaitStatus::Exited(code) => code,
        WaitStatus::Signaled(sig) => {
            eprintln!("rsh: killed by signal {}", sig);
            128 + sig
        }
        WaitStatus::Stopped(sig) => 128 + sig,
    }
}

fn wait_all(pids: &[pid_t]) {
    for pid in pids {
        wait_for(*pid);
    }
}

fn close_fd(fd: Option<c_int>) {
    if let Some(fd) = fd {
        unsafe { close(fd) };
    }
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
