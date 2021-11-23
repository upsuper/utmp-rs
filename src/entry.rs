use chrono::{DateTime, TimeZone, Utc};
use libc::pid_t;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::os::raw::c_short;
use thiserror::Error;
use utmp_raw::{timeval, utmp};

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UtmpEntry {
    /// Record does not contain valid info
    Empty,
    /// Change in system run-level (see `init(8)`)
    RunLevel {
        pid: pid_t,
        /// Kernel version
        kernel_version: String,
        /// Time entry was made
        time: DateTime<Utc>,
        user: String,
    },
    /// Time of system boot
    BootTime(DateTime<Utc>),
    /// Time after system clock change
    NewTime(DateTime<Utc>),
    /// Time before system clock change
    OldTime(DateTime<Utc>),
    /// Process spawned by `init(8)`
    InitProcess {
        /// PID of the init process
        pid: pid_t,
        /// Time entry was made
        time: DateTime<Utc>,
    },
    /// Session leader process for user login
    LoginProcess {
        /// PID of the login process
        pid: pid_t,
        /// Time entry was made
        time: DateTime<Utc>,
    },
    /// Normal process
    UserProcess {
        /// PID of login process
        pid: pid_t,
        /// Device name of tty
        line: String,
        /// Username
        user: String,
        /// Hostname for remote login
        host: String,
        /// Session ID (`getsid(2)`)
        session: pid_t,
        /// Time entry was made
        time: DateTime<Utc>,
        // TODO: Figure out the correct byte order to parse the address
        // address: IpAddr,
    },
    /// Terminated process
    DeadProcess {
        /// PID of the terminated process
        pid: pid_t,
        /// Device name of tty
        line: String,
        /// Time entry was made
        time: DateTime<Utc>,
    },
    /// Not implemented
    #[non_exhaustive]
    Accounting,
}

impl<'a> TryFrom<&'a utmp> for UtmpEntry {
    type Error = UtmpError;

    fn try_from(from: &utmp) -> Result<Self, UtmpError> {
        Ok(match from.ut_type {
            utmp_raw::EMPTY => UtmpEntry::Empty,
            utmp_raw::RUN_LVL => UtmpEntry::RunLevel {
                pid: from.ut_pid,
                kernel_version: string_from_bytes(&from.ut_host).map_err(UtmpError::InvalidHost)?,
                time: time_from_tv(from.ut_tv)?,
                user: string_from_bytes(&from.ut_user).map_err(UtmpError::InvalidUser)?,
            },
            utmp_raw::BOOT_TIME => UtmpEntry::BootTime(time_from_tv(from.ut_tv)?),
            utmp_raw::NEW_TIME => UtmpEntry::NewTime(time_from_tv(from.ut_tv)?),
            utmp_raw::OLD_TIME => UtmpEntry::OldTime(time_from_tv(from.ut_tv)?),
            utmp_raw::INIT_PROCESS => UtmpEntry::InitProcess {
                pid: from.ut_pid,
                time: time_from_tv(from.ut_tv)?,
            },
            utmp_raw::LOGIN_PROCESS => UtmpEntry::LoginProcess {
                pid: from.ut_pid,
                time: time_from_tv(from.ut_tv)?,
            },
            utmp_raw::USER_PROCESS => UtmpEntry::UserProcess {
                pid: from.ut_pid,
                line: string_from_bytes(&from.ut_line).map_err(UtmpError::InvalidLine)?,
                user: string_from_bytes(&from.ut_user).map_err(UtmpError::InvalidUser)?,
                host: string_from_bytes(&from.ut_host).map_err(UtmpError::InvalidHost)?,
                session: from.ut_session,
                time: time_from_tv(from.ut_tv)?,
            },
            utmp_raw::DEAD_PROCESS => UtmpEntry::DeadProcess {
                pid: from.ut_pid,
                line: string_from_bytes(&from.ut_line).map_err(UtmpError::InvalidLine)?,
                time: time_from_tv(from.ut_tv)?,
            },
            utmp_raw::ACCOUNTING => UtmpEntry::Accounting,
            _ => return Err(UtmpError::UnknownType(from.ut_type)),
        })
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UtmpError {
    #[error("unknown type {0}")]
    UnknownType(c_short),
    #[error("invalid time value {0:?}")]
    InvalidTime(timeval),
    #[error("invalid line value `{0:?}`")]
    InvalidLine(Box<[u8]>),
    #[error("invalid user value `{0:?}`")]
    InvalidUser(Box<[u8]>),
    #[error("invalid host value `{0:?}`")]
    InvalidHost(Box<[u8]>),
}

fn time_from_tv(tv: timeval) -> Result<DateTime<Utc>, UtmpError> {
    let timeval { tv_sec, tv_usec } = tv;
    match tv_usec {
        usec if usec < 0 => Err(UtmpError::InvalidTime(tv)),
        usec => Ok(Utc.timestamp(i64::from(tv_sec), usec as u32 * 1000)),
    }
}

fn string_from_bytes(bytes: &[u8]) -> Result<String, Box<[u8]>> {
    bytes
        .iter()
        .position(|b| *b == 0)
        .and_then(|pos| {
            // This is safe because we manually located the first zero byte above.
            let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(&bytes[..=pos]) };
            Some(cstr.to_str().ok()?.to_string())
        })
        .ok_or_else(|| bytes.to_owned().into_boxed_slice())
}
