use crate::{UtmpEntry, UtmpError};
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use thiserror::Error;
use utmp_raw::utmp;
use zerocopy::LayoutVerified;

const UTMP_SIZE: usize = std::mem::size_of::<utmp>();

#[repr(align(4))]
struct UtmpBuffer([u8; UTMP_SIZE]);

/// Parser to parse a utmp file. It can be used as an iterator.
///
/// ```
/// # use utmp_rs::UtmpParser;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// for entry in UtmpParser::from_path("/var/run/utmp")? {
///     let entry = entry?;
///     // handle entry
/// }
/// # Ok(())
/// # }
/// ```
pub struct UtmpParser<R>(R);

impl<R: Read> UtmpParser<R> {
    pub fn from_reader(reader: R) -> Self {
        UtmpParser(reader)
    }

    pub fn into_inner(self) -> R {
        self.0
    }
}

impl UtmpParser<BufReader<File>> {
    pub fn from_file(file: File) -> Self {
        UtmpParser(BufReader::new(file))
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        Ok(Self::from_file(File::open(path)?))
    }
}

impl<R: Read> Iterator for UtmpParser<R> {
    type Item = Result<UtmpEntry, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = UtmpBuffer([0; UTMP_SIZE]);
        let mut buf = buffer.0.as_mut();
        loop {
            match self.0.read(buf) {
                // If the buffer has not been filled, then we just passed the last item.
                Ok(0) if buf.len() == UTMP_SIZE => return None,
                // Otherwise this is an unexpected EOF.
                Ok(0) => {
                    let inner = io::Error::new(io::ErrorKind::UnexpectedEof, "size not aligned");
                    return Some(Err(inner.into()));
                }
                Ok(n) => {
                    buf = &mut buf[n..];
                    if buf.is_empty() {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Some(Err(e.into())),
            }
        }
        let buffer = buffer.0.as_ref();
        let entry = LayoutVerified::<_, utmp>::new(buffer).unwrap().into_ref();
        Some(UtmpEntry::try_from(entry).map_err(ParseError::Utmp))
    }
}

/// Parse utmp entries from the given path.
pub fn parse_from_path<P: AsRef<Path>>(path: P) -> Result<Vec<UtmpEntry>, ParseError> {
    UtmpParser::from_path(path)?.collect()
}

/// Parse utmp entries from the given file.
pub fn parse_from_file(file: File) -> Result<Vec<UtmpEntry>, ParseError> {
    UtmpParser::from_file(file).collect()
}

/// Parse utmp entries from the given reader.
pub fn parse_from_reader<R: Read>(reader: R) -> Result<Vec<UtmpEntry>, ParseError> {
    UtmpParser::from_reader(reader).collect()
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error(transparent)]
    Utmp(#[from] UtmpError),
    #[error(transparent)]
    Io(#[from] io::Error),
}
