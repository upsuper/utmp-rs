//! A Rust crate for parsing `utmp` files like `/var/run/utmp` and `/var/log/wtmp`.
//!
//! ## Usage
//!
//! ```rust
//! # use anyhow::Result;
//! fn main() -> Result<()> {
//!     let entries = utmp_rs::parse_from_path("/var/run/utmp")?;
//!     // ...
//! #   Ok(())
//! }
//! ```

mod entry;
mod parse;

pub use entry::{UtmpEntry, UtmpError};
pub use parse::{parse_from_file, parse_from_path, parse_from_reader};
pub use parse::{ParseError, Utmp32Parser, Utmp64Parser, UtmpParser};
