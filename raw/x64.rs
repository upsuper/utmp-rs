use super::{cstr_from_bytes, exit_status, UT_HOSTSIZE, UT_LINESIZE, UT_NAMESIZE};
use libc::c_short;
use std::fmt;
use zerocopy::FromBytes;

#[repr(C)]
#[derive(Clone, Copy, Debug, FromBytes)]
pub struct timeval {
    /// Seconds
    pub tv_sec: i64,
    /// Microseconds
    pub tv_usec: i64,
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
    pub ut_session: i64,
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

#[test]
fn test_size_of_utmp_x64() {
    assert_eq!(std::mem::size_of::<utmp>(), 400);
}
