use std::ffi::{c_char, c_int};

#[allow(non_camel_case_types)]
type pid_t = i32;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pid() {
        let result = unsafe { getpid() };
        let expected: i32 = std::process::id().try_into().unwrap();
        assert_eq!(expected, result);
    }
}
