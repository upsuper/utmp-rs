use crate::{UtmpEntry, UtmpError};
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use thiserror::Error;
use utmp_raw::{utmp, x32::utmp as utmp32, x64::utmp as utmp64};
use zerocopy::{FromBytes, LayoutVerified};

/// Parser to parse a utmp file. It can be used as an iterator.
///
/// ```
/// use utmp_raw::utmp;
/// # use utmp_rs::UtmpParser;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// for entry in UtmpParser::<_, utmp>::from_path("/var/run/utmp")? {
///     let entry = entry?;
///     // handle entry
/// }
/// # Ok(())
/// # }
/// ```
pub struct UtmpParser<R, T = utmp>(R, PhantomData<T>);

impl<R: Read, T> UtmpParser<R, T> {
    pub fn from_reader(reader: R) -> Self {
        UtmpParser(reader, PhantomData)
    }

    pub fn into_inner(self) -> R {
        self.0
    }
}

impl<T> UtmpParser<BufReader<File>, T> {
    pub fn from_file(file: File) -> Self {
        UtmpParser(BufReader::new(file), PhantomData)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        Ok(Self::from_file(File::open(path)?))
    }
}

const UTMP32_SIZE: usize = mem::size_of::<utmp32>();
const UTMP64_SIZE: usize = mem::size_of::<utmp64>();

impl<R: Read> Iterator for UtmpParser<R, utmp32> {
    type Item = Result<UtmpEntry, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        #[repr(align(4))]
        struct Buffer([u8; UTMP32_SIZE]);
        let mut buffer = Buffer([0; UTMP32_SIZE]);
        match read_entry::<_, utmp32>(&mut self.0, buffer.0.as_mut()) {
            Ok(None) => None,
            Ok(Some(entry)) => Some(UtmpEntry::try_from(entry).map_err(ParseError::Utmp)),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<R: Read> Iterator for UtmpParser<R, utmp64> {
    type Item = Result<UtmpEntry, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        #[repr(align(8))]
        struct Buffer([u8; UTMP64_SIZE]);
        let mut buffer = Buffer([0; UTMP64_SIZE]);
        match read_entry::<_, utmp64>(&mut self.0, buffer.0.as_mut()) {
            Ok(None) => None,
            Ok(Some(entry)) => Some(UtmpEntry::try_from(entry).map_err(ParseError::Utmp)),
            Err(e) => Some(Err(e)),
        }
    }
}

fn read_entry<R: Read, T: FromBytes>(
    mut reader: R,
    buffer: &mut [u8],
) -> Result<Option<&T>, ParseError> {
    let size = buffer.len();
    let mut buf = &mut buffer[..];
    loop {
        match reader.read(buf) {
            // If the buffer has not been filled, then we just passed the last item.
            Ok(0) if buf.len() == size => return Ok(None),
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
    Ok(Some(
        LayoutVerified::<_, T>::new(buffer).unwrap().into_ref(),
    ))
}

/// Parse utmp entries from the given path.
///
/// It parses the given path using the native utmp format in the target platform.
pub fn parse_from_path<P: AsRef<Path>>(path: P) -> Result<Vec<UtmpEntry>, ParseError> {
    UtmpParser::<_, utmp>::from_path(path)?.collect()
}

/// Parse utmp entries from the given file.
///
/// It parses the given file using the native utmp format in the target platform.
pub fn parse_from_file(file: File) -> Result<Vec<UtmpEntry>, ParseError> {
    UtmpParser::<_, utmp>::from_file(file).collect()
}

/// Parse utmp entries from the given reader.
///
/// It parses from the given reader using the native utmp format in the target platform.
pub fn parse_from_reader<R: Read>(reader: R) -> Result<Vec<UtmpEntry>, ParseError> {
    UtmpParser::<_, utmp>::from_reader(reader).collect()
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    #[error(transparent)]
    Utmp(#[from] UtmpError),
    #[error(transparent)]
    Io(#[from] io::Error),
}
