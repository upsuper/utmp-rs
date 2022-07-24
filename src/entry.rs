use libc::pid_t;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::os::raw::c_short;
use thiserror::Error;
use time::OffsetDateTime;
use utmp_raw::x32::utmp as utmp32;
use utmp_raw::x64::{timeval as timeval64, utmp as utmp64};

/// Parsed utmp entry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UtmpEntry {
    /// Record does not contain valid info
    Empty,
    /// Change in system run-level (see `init(8)`)
    RunLevel {
        /// Kernel version
        kernel_version: String,
        /// Time entry was made
        time: OffsetDateTime,
    },
    /// Time of system boot
    BootTime {
        /// Kernel version
        kernel_version: String,
        /// Time entry was made
        time: OffsetDateTime,
    },
    /// Time of system shutdown
    ShutdownTime {
        /// Kernel version
        kernel_version: String,
        /// Time entry was made
        time: OffsetDateTime,
    },
    /// Time after system clock change
    NewTime(OffsetDateTime),
    /// Time before system clock change
    OldTime(OffsetDateTime),
    /// Process spawned by `init(8)`
    InitProcess {
        /// PID of the init process
        pid: pid_t,
        /// Time entry was made
        time: OffsetDateTime,
    },
    /// Session leader process for user login
    LoginProcess {
        /// PID of the login process
        pid: pid_t,
        /// Time entry was made
        time: OffsetDateTime,
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
        time: OffsetDateTime,
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
        time: OffsetDateTime,
    },
    /// Not implemented
    #[non_exhaustive]
    Accounting,
}

impl<'a> TryFrom<&'a utmp32> for UtmpEntry {
    type Error = UtmpError;

    fn try_from(from: &utmp32) -> Result<Self, UtmpError> {
        UtmpEntry::try_from(&utmp64 {
            ut_type: from.ut_type,
            ut_pid: from.ut_pid,
            ut_line: from.ut_line,
            ut_id: from.ut_id,
            ut_user: from.ut_user,
            ut_host: from.ut_host,
            ut_exit: from.ut_exit,
            ut_session: i64::from(from.ut_session),
            ut_tv: timeval64 {
                tv_sec: i64::from(from.ut_tv.tv_sec),
                tv_usec: i64::from(from.ut_tv.tv_usec),
            },
            ut_addr_v6: from.ut_addr_v6,
            __unused: from.__unused,
        })
    }
}

impl<'a> TryFrom<&'a utmp64> for UtmpEntry {
    type Error = UtmpError;

    fn try_from(from: &utmp64) -> Result<Self, UtmpError> {
        Ok(match from.ut_type {
            utmp_raw::EMPTY => UtmpEntry::Empty,
            utmp_raw::RUN_LVL => {
                let kernel_version =
                    string_from_bytes(&from.ut_host).map_err(UtmpError::InvalidHost)?;
                let time = time_from_tv(from.ut_tv)?;
                if from.ut_line[0] == b'~' && from.ut_user.starts_with(b"shutdown\0") {
                    UtmpEntry::ShutdownTime {
                        kernel_version,
                        time,
                    }
                } else {
                    UtmpEntry::RunLevel {
                        kernel_version,
                        time,
                    }
                }
            }
            utmp_raw::BOOT_TIME => UtmpEntry::BootTime {
                kernel_version: string_from_bytes(&from.ut_host).map_err(UtmpError::InvalidHost)?,
                time: time_from_tv(from.ut_tv)?,
            },
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
                session: from.ut_session as pid_t,
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
    InvalidTime(timeval64),
    #[error("invalid line value `{0:?}`")]
    InvalidLine(Box<[u8]>),
    #[error("invalid user value `{0:?}`")]
    InvalidUser(Box<[u8]>),
    #[error("invalid host value `{0:?}`")]
    InvalidHost(Box<[u8]>),
}

fn time_from_tv(tv: timeval64) -> Result<OffsetDateTime, UtmpError> {
    let timeval64 { tv_sec, tv_usec } = tv;
    if tv_usec < 0 {
        return Err(UtmpError::InvalidTime(tv));
    }
    let usec = i128::from(tv_sec) * 1_000_000 + i128::from(tv_usec);
    OffsetDateTime::from_unix_timestamp_nanos(usec * 1000).map_err(|_| UtmpError::InvalidTime(tv))
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
