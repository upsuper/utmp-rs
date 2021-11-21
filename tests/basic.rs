use anyhow::Result;
use chrono::{TimeZone, Utc};
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::iter::FromIterator;
use std::path::PathBuf;
use utmp_rs::{parse_from_path, parse_from_reader, UtmpEntry};

static SAMPLES_PATH: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from_iter(&[env!("CARGO_MANIFEST_DIR"), "tests", "samples"]));

fn get_basic_expected() -> Vec<UtmpEntry> {
    vec![
        UtmpEntry::BootTime(Utc.timestamp(1581199438, 54727000)),
        UtmpEntry::RunLevel {
            kernel_version: "5.3.0-29-generic".to_owned(),
            time: Utc.timestamp(1581199447, 558900000),
        },
        UtmpEntry::UserProcess {
            pid: 2555,
            line: ":1".to_owned(),
            user: "upsuper".to_owned(),
            host: ":1".to_owned(),
            session: 0,
            time: Utc.timestamp(1581199675, 609322000),
        },
        UtmpEntry::UserProcess {
            pid: 28885,
            line: "tty3".to_owned(),
            user: "upsuper".to_owned(),
            host: "".to_owned(),
            session: 28786,
            time: Utc.timestamp(1581217267, 195722000),
        },
        UtmpEntry::LoginProcess {
            pid: 28965,
            time: Utc.timestamp(1581217268, 463588000),
        },
    ]
}

#[test]
fn parse_basic() -> Result<()> {
    let path = SAMPLES_PATH.join("basic.utmp");
    let actual = parse_from_path(&path)?;
    let expected = get_basic_expected();
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
    let path = SAMPLES_PATH.join("basic.utmp");
    let reader = ByteReader(BufReader::new(File::open(&path)?));
    let actual = parse_from_reader(reader)?;
    let expected = get_basic_expected();
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
