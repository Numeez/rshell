use std::ffi::{c_char, c_int};

#[allow(non_camel_case_types)]
type pid_t = i32;

const STATUS_MASK: c_int = 0x7F;
const EXIT_CODE_MASK: c_int = 0xFF;

#[derive(PartialEq, Debug)]
pub enum WaitStatus {
    Exited(c_int),
    Stopped(c_int),
    Signaled(c_int),
}

unsafe extern "C" {
    pub fn getpid() -> pid_t;
}

unsafe extern "C" {
    pub fn fork() -> pid_t;
}

unsafe extern "C" {
    pub fn waitpid(pid: pid_t, stat_loc: *mut i32, options: i32) -> pid_t;
}

unsafe extern "C" {
    pub fn execvp(file: *const c_char, argv: *const *const c_char) -> c_int;
}
unsafe extern "C" {
    pub fn _exit(status: i32) -> !;
}

pub fn get_status(code: c_int) -> WaitStatus {
    let result_code = get_status_code(code);
    match result_code {
        0 => WaitStatus::Exited(get_exit_code(code)),
        0x7F => WaitStatus::Stopped(get_exit_code(code)),
        _ => WaitStatus::Signaled(result_code),
    }
}

fn get_exit_code(code: c_int) -> c_int {
    (code >> 8) & EXIT_CODE_MASK
}
fn get_status_code(code: c_int) -> c_int {
    code & STATUS_MASK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pid() {
        let result = unsafe { getpid() };
        let expected: i32 = std::process::id().try_into().unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn decodes_exit_code_zero() {
        assert_eq!(get_status(0), WaitStatus::Exited(0));
    }

    #[test]
    fn decodes_exit_code_one() {
        assert_eq!(get_status(1 << 8), WaitStatus::Exited(1));
    }

    #[test]
    fn decodes_largest_exit_code() {
        assert_eq!(get_status(255 << 8), WaitStatus::Exited(255));
    }

    #[test]
    fn decodes_killed_by_signal() {
        assert_eq!(get_status(9), WaitStatus::Signaled(9));
    }

    #[test]
    fn decodes_stopped_by_signal() {
        assert_eq!(get_status((18 << 8) | 0x7F), WaitStatus::Stopped(18));
    }
}
