mod entry;
mod parse;

pub use entry::{UtmpEntry, UtmpError};
pub use parse::{parse_from_file, parse_from_path, parse_from_reader, ParseError};
