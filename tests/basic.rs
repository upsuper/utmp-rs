use anyhow::Result;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::iter::FromIterator;
use std::path::PathBuf;
use time::OffsetDateTime;
use utmp_rs::{parse_from_path, Utmp32Parser, Utmp64Parser, UtmpEntry};

static SAMPLES_PATH: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from_iter(&[env!("CARGO_MANIFEST_DIR"), "tests", "samples"]));

fn timestamp(nanos: i128) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap()
}

fn get_basic32_expected() -> Vec<UtmpEntry> {
    vec![
        UtmpEntry::BootTime {
            kernel_version: "5.3.0-29-generic".to_owned(),
            time: timestamp(1581199438_054727_000),
        },
        UtmpEntry::RunLevel {
            pid: 53,
            kernel_version: "5.3.0-29-generic".to_owned(),
            time: timestamp(1581199447_558900_000),
        },
        UtmpEntry::UserProcess {
            pid: 2555,
            line: ":1".to_owned(),
            user: "upsuper".to_owned(),
            host: ":1".to_owned(),
            session: 0,
            time: timestamp(1581199675_609322_000),
        },
        UtmpEntry::UserProcess {
            pid: 28885,
            line: "tty3".to_owned(),
            user: "upsuper".to_owned(),
            host: "".to_owned(),
            session: 28786,
            time: timestamp(1581217267_195722_000),
        },
        UtmpEntry::LoginProcess {
            pid: 28965,
            time: timestamp(1581217268_463588_000),
            line: "tty4".to_owned(),
            user: "LOGIN".to_owned(),
            host: "".to_owned(),
        },
    ]
}

fn get_with_host_32_expected() -> Vec<UtmpEntry> {
    vec![
        UtmpEntry::ShutdownTime {
            kernel_version: "5.4.0-135-generic".to_owned(),
            time: timestamp(1672223597_077918_000),
        },
        UtmpEntry::BootTime {
            kernel_version: "5.4.0-135-generic".to_owned(),
            time: timestamp(1675756860_150698_000),
        },
        UtmpEntry::RunLevel {
            pid: 53,
            kernel_version: "5.4.0-135-generic".to_owned(),
            time: timestamp(1675756874_594747_000),
        },
        UtmpEntry::InitProcess {
            pid: 627,
            time: timestamp(1675756875_303010_000),
        },
        UtmpEntry::InitProcess {
            pid: 644,
            time: timestamp(1675756875_305313_000),
        },
        UtmpEntry::LoginProcess {
            pid: 644,
            line: "tty1".to_owned(),
            user: "LOGIN".to_owned(),
            host: "".to_owned(),
            time: timestamp(1675756875_305313_000),
        },
        UtmpEntry::LoginProcess {
            pid: 627,
            line: "ttyS0".to_owned(),
            user: "LOGIN".to_owned(),
            host: "".to_owned(),
            time: timestamp(1675756875_303010_000),
        },
        UtmpEntry::UserProcess {
            pid: 1125,
            line: "pts/0".to_owned(),
            user: "root".to_owned(),
            host: "112.124.2.209".to_owned(),
            session: 0,
            time: timestamp(1675757226_139552_000),
        },
        UtmpEntry::UserProcess {
            pid: 1127,
            line: "pts/1".to_owned(),
            user: "root".to_owned(),
            host: "112.124.2.209".to_owned(),
            session: 0,
            time: timestamp(1675757226_284647_000),
        },
        UtmpEntry::DeadProcess {
            pid: 1020,
            line: "pts/0".to_owned(),
            time: timestamp(1675757226_404205_000),
        },
        UtmpEntry::DeadProcess {
            pid: 1020,
            line: "pts/1".to_owned(),
            time: timestamp(1675757227_275375_000),
        },
        UtmpEntry::UserProcess {
            pid: 1225,
            line: "pts/0".to_owned(),
            user: "root".to_owned(),
            host: "112.124.2.209".to_owned(),
            session: 0,
            time: timestamp(1675757312_920719_000),
        },
        UtmpEntry::UserProcess {
            pid: 2454,
            line: "pts/1".to_owned(),
            user: "root".to_owned(),
            host: "".to_owned(),
            session: 0,
            time: timestamp(1675758317_098468_000),
        },
        UtmpEntry::UserProcess {
            pid: 2714,
            line: "pts/1".to_owned(),
            user: "root".to_owned(),
            host: "".to_owned(),
            session: 0,
            time: timestamp(1675758522_887514_000),
        },
        UtmpEntry::DeadProcess {
            pid: 1189,
            line: "pts/0".to_owned(),
            time: timestamp(1675759743_147069_000),
        },
        UtmpEntry::UserProcess {
            pid: 4343,
            line: "pts/0".to_owned(),
            user: "root".to_owned(),
            host: "112.124.2.209".to_owned(),
            session: 0,
            time: timestamp(1675759955_391532_000),
        },
        UtmpEntry::UserProcess {
            pid: 5022,
            line: "pts/1".to_owned(),
            user: "root".to_owned(),
            host: "".to_owned(),
            session: 0,
            time: timestamp(1675760619_783753_000),
        },
        UtmpEntry::DeadProcess {
            pid: 4305,
            line: "pts/0".to_owned(),
            time: timestamp(1675761785_613258_000),
        },
        UtmpEntry::UserProcess {
            pid: 13369,
            line: "pts/0".to_owned(),
            user: "root".to_owned(),
            host: "112.124.2.209".to_owned(),
            session: 0,
            time: timestamp(1675768806_832709_000),
        },
    ]
}

fn get_long_user_32_expected() -> Vec<UtmpEntry> {
    vec![
        UtmpEntry::LoginProcess {
            pid: 1872475,
            line: "pts/1".to_owned(),
            user: "abc".to_owned(),
            host: "".to_owned(),
            time: timestamp(1675278673_563046_000),
        },
        UtmpEntry::LoginProcess {
            pid: 1874257,
            line: "pts/1".to_owned(),
            user: "abc".to_owned(),
            host: "".to_owned(),
            time: timestamp(1675278942_329935_000),
        },
        UtmpEntry::LoginProcess {
            pid: 1875352,
            line: "ssh:notty".to_owned(),
            user: "abc".to_owned(),
            host: "10.11.0.169".to_owned(),
            time: timestamp(1675279200_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 1875352,
            line: "ssh:notty".to_owned(),
            user: "abc".to_owned(),
            host: "10.11.0.169".to_owned(),
            time: timestamp(1675279205_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 1875352,
            line: "ssh:notty".to_owned(),
            user: "abc".to_owned(),
            host: "10.11.0.169".to_owned(),
            time: timestamp(1675279206_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2199784,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675423140_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2199784,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675423143_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2199784,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675423148_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2200630,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675423317_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2200630,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675423321_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2200630,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675423325_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2200630,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675423330_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2203029,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675424016_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2203029,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675424020_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2203029,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675424024_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2203029,
            line: "ssh:notty".to_owned(),
            user: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675424031_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2214635,
            line: "ssh:notty".to_owned(),
            user: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675424626_000000_000),
        },
        UtmpEntry::LoginProcess {
            pid: 2214635,
            line: "ssh:notty".to_owned(),
            user: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
            host: "10.10.4.230".to_owned(),
            time: timestamp(1675424630_000000_000),
        },
    ]
}

fn get_basic64_expected() -> Vec<UtmpEntry> {
    vec![
        UtmpEntry::BootTime {
            kernel_version: "5.15.0-41-generic".to_owned(),
            time: timestamp(1658083371_314869_000),
        },
        UtmpEntry::RunLevel {
            pid: 53,
            kernel_version: "5.15.0-41-generic".to_owned(),
            time: timestamp(1658083400_855073_000),
        },
        UtmpEntry::LoginProcess {
            pid: 1219,
            time: timestamp(1658083400_866391_000),
            line: "ttyAMA0".to_owned(),
            user: "LOGIN".to_owned(),
            host: "".to_owned(),
        },
    ]
}

#[test]
fn parse_basic32() -> Result<()> {
    let path = SAMPLES_PATH.join("basic32.utmp");
    let actual = Utmp32Parser::from_path(&path)?.collect::<Result<Vec<_>, _>>()?;
    let expected = get_basic32_expected();
    Ok(assert_eq!(actual, expected))
}

#[test]
fn parse_with_host_32() -> Result<()> {
    let path = SAMPLES_PATH.join("with_host_32.utmp");
    let actual = Utmp32Parser::from_path(&path)?.collect::<Result<Vec<_>, _>>()?;
    let expected = get_with_host_32_expected();
    Ok(assert_eq!(actual, expected))
}

#[test]
fn parse_long_user_32() -> Result<()> {
    let path = SAMPLES_PATH.join("long_user_32.utmp");
    let actual = Utmp32Parser::from_path(&path)?.collect::<Result<Vec<_>, _>>()?;
    let expected = get_long_user_32_expected();
    Ok(assert_eq!(actual, expected))
}

#[test]
fn parse_basic64() -> Result<()> {
    let path = SAMPLES_PATH.join("basic64.utmp");
    let actual = Utmp64Parser::from_path(&path)?.collect::<Result<Vec<_>, _>>()?;
    let expected = get_basic64_expected();
    Ok(assert_eq!(actual, expected))
}

#[test]
fn parse_empty() -> Result<()> {
    let path = SAMPLES_PATH.join("empty.utmp");
    let actual = parse_from_path(&path)?;
    let expected = vec![];
    Ok(assert_eq!(actual, expected))
}

#[test]
fn parse_with_partial_read() -> Result<()> {
    let path = SAMPLES_PATH.join("basic32.utmp");
    let reader = ByteReader(BufReader::new(File::open(&path)?));
    let actual = Utmp32Parser::from_reader(reader).collect::<Result<Vec<_>, _>>()?;
    let expected = get_basic32_expected();
    Ok(assert_eq!(actual, expected))
}

struct ByteReader<R>(R);

impl<R: Read> Read for ByteReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 1 {
            self.0.read(buf)
        } else {
            self.0.read(&mut buf[..1])
        }
    }
}
