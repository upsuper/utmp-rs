#![allow(non_camel_case_types)]

use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_short;
use zerocopy::FromBytes;

/// Record does not contain valid info (formerly known as `UT_UNKNOWN` on Linux)
pub const EMPTY: c_short = 0;
/// Change in system run-level (see `init(8)`)
pub const RUN_LVL: c_short = 1;
/// Time of system boot (in `ut_tv`)
pub const BOOT_TIME: c_short = 2;
/// Time after system clock change (in `ut_tv`)
pub const NEW_TIME: c_short = 3;
/// Time before system clock change (in `ut_tv`)
pub const OLD_TIME: c_short = 4;
/// Process spawned by `init(8)`
pub const INIT_PROCESS: c_short = 5;
/// Session leader process for user login
pub const LOGIN_PROCESS: c_short = 6;
/// Normal process
pub const USER_PROCESS: c_short = 7;
/// Terminated process
pub const DEAD_PROCESS: c_short = 8;
/// Not implemented
pub const ACCOUNTING: c_short = 9;

pub const UT_LINESIZE: usize = 32;
pub const UT_NAMESIZE: usize = 32;
pub const UT_HOSTSIZE: usize = 256;

/// Type for `ut_exit`, below
#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes)]
pub struct exit_status {
    /// Process termination status
    pub e_termination: c_short,
    /// Process exit status
    pub e_exit: c_short,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes)]
pub struct timeval {
    /// Seconds
    pub tv_sec: i32,
    /// Microseconds
    pub tv_usec: i32,
}

#[repr(C)]
#[derive(Clone, Copy, FromBytes)]
pub struct utmp {
    /// Type of record
    pub ut_type: c_short,
    /// PID of login process
    pub ut_pid: libc::pid_t,
    /// Device name of tty - `"/dev/"`
    pub ut_line: [u8; UT_LINESIZE],
    /// Terminal name suffix, or `inittab(5)` ID
    pub ut_id: [u8; 4],
    /// Username
    pub ut_user: [u8; UT_NAMESIZE],
    /// Hostname for remote login, or kernel version for run-level message
    pub ut_host: [u8; UT_HOSTSIZE],
    /// Exit status of a process marked as `DEAD_PROCESS`; not used by Linux init
    pub ut_exit: exit_status,
    /// Session ID (`getsid(2)`) used for windowing
    pub ut_session: i32,
    /// Time entry was made
    pub ut_tv: timeval,
    /// Internet address of remote host; IPv4 address uses just `ut_addr_v6[0]`
    pub ut_addr_v6: [i32; 4],
    /// Reserved for future use
    pub __unused: [u8; 20],
}

impl fmt::Debug for utmp {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("utmp")
            .field("ut_type", &self.ut_type)
            .field("ut_pid", &self.ut_pid)
            .field("ut_line", &cstr_from_bytes(&self.ut_line))
            .field("ut_id", &self.ut_id)
            .field("ut_user", &cstr_from_bytes(&self.ut_user))
            .field("ut_host", &cstr_from_bytes(&self.ut_host))
            .field("ut_exit", &self.ut_exit)
            .field("ut_session", &self.ut_session)
            .field("ut_tv", &self.ut_tv)
            .field("ut_addr_v6", &self.ut_addr_v6)
            .field("__unused", &self.__unused)
            .finish()
    }
}

fn cstr_from_bytes(bytes: &[u8]) -> &CStr {
    match bytes.iter().position(|b| *b == 0) {
        // This is safe because we manually located the first zero byte above.
        Some(pos) => unsafe { CStr::from_bytes_with_nul_unchecked(&bytes[..=pos]) },
        // This is safe because we manually generated this string.
        None => unsafe { CStr::from_bytes_with_nul_unchecked("???\0".as_bytes()) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_size_of() {
        assert_eq!(mem::size_of::<utmp>(), 384);
    }
}
