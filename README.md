# utmp-rs

[![Crates.io](https://img.shields.io/crates/v/utmp-rs.svg)](https://crates.io/crates/utmp-rs)
[![Docs](https://docs.rs/utmp-rs/badge.svg)](https://docs.rs/utmp-rs)

Rust crate for parsing `utmp` files like `/var/run/utmp` and `/var/log/wtmp`.

## Usage

```rust
fn main() -> Result<()> {
    let entries = utmp_rs::parse_from_path("/var/run/utmp")?;
    // ...
}
```
