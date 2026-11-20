use std::ffi::c_int;

const STATUS_MASK: c_int = 0x7F;
const EXIT_CODE_MASK: c_int = 0xFF;

#[derive(PartialEq, Debug)]
pub enum WaitStatus {
    Exited(c_int),
    Stopped(c_int),
    Signaled(c_int),
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
mod test {
    use super::*;
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
