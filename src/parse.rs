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

/// Parse utmp entries from the given path.
pub fn parse_from_path<P: AsRef<Path>>(path: P) -> Result<Vec<UtmpEntry>, ParseError> {
    parse_from_file(File::open(path)?)
}

/// Parse utmp entries from the given file.
pub fn parse_from_file(file: File) -> Result<Vec<UtmpEntry>, ParseError> {
    parse_from_reader(BufReader::new(file))
}

/// Parse utmp entries from the given reader.
pub fn parse_from_reader<R: Read>(mut reader: R) -> Result<Vec<UtmpEntry>, ParseError> {
    let mut result = Vec::new();
    let mut buffer = UtmpBuffer([0; UTMP_SIZE]);
    'outer: loop {
        loop {
            let mut buf = buffer.0.as_mut();
            match reader.read(buf) {
                // If the buffer has not been filled, then we just passed the last item.
                Ok(0) if buf.len() == UTMP_SIZE => break 'outer,
                // Otherwise this is an unexpected EOF.
                Ok(0) => {
                    let inner = io::Error::new(io::ErrorKind::UnexpectedEof, "size not aligned");
                    return Err(inner.into());
                }
                Ok(n) => {
                    buf = &mut buf[n..];
                    if buf.is_empty() {
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e.into()),
            }
        }
        let buffer = buffer.0.as_ref();
        let entry = LayoutVerified::<_, utmp>::new(buffer).unwrap().into_ref();
        result.push(UtmpEntry::try_from(entry)?);
    }
    Ok(result)
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error(transparent)]
    Utmp(#[from] UtmpError),
    #[error(transparent)]
    Io(#[from] io::Error),
}
