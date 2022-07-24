# utmp-rs

[![Crates.io](https://img.shields.io/crates/v/utmp-rs.svg)](https://crates.io/crates/utmp-rs)
[![Docs](https://docs.rs/utmp-rs/badge.svg)](https://docs.rs/utmp-rs)

<!-- cargo-sync-readme start -->

A Rust crate for parsing `utmp` files like `/var/run/utmp` and `/var/log/wtmp`.

## Usage

Simplest way is to use `parse_from_*` functions,
which returns a `Vec<UtmpEntry>` on success:
```rust
let entries = utmp_rs::parse_from_path("/var/run/utmp")?;
// ...
```

If you don't need to collect them all,
`UtmpParser` can be used as an iterator:
```rust
use utmp_rs::UtmpParser;
for entry in UtmpParser::from_path("/var/run/utmp")? {
    let entry = entry?;
    // ...
}
```

All the `parse_from_*` functions as well as `UtmpParser` parse `utmp` file
based on the native format for the target platform.
If cross-platform parsing is needed,
`Utmp32Parser` or `Utmp64Parser` can be used instead of `UtmpParser`.

<!-- cargo-sync-readme end -->
