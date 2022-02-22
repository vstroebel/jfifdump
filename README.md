# JFIF-Dump

[![docs.rs badge](https://docs.rs/jfifdump/badge.svg)](https://docs.rs/jfifdump/latest/jfifdump/)
[![crates.io badge](https://img.shields.io/crates/v/jfifdump.svg)](https://crates.io/crates/jfifdump/)
[![Rust](https://github.com/vstroebel/jfifdump/actions/workflows/rust.yml/badge.svg)](https://github.com/vstroebel/jfifdump/actions/workflows/rust.yml)

Read and dump structure of a jpeg file.

This crate can be used as a library or as a command line utility.

## Installation

```
$ cargo install jfifdump
```

## Usage

```
$ jfifdump image.jpeg
```

## Command-line options

```
Read and dump structure of a jpeg file

USAGE:
    jfifdump [OPTIONS] <INPUT>

ARGS:
    <INPUT>    Jpeg file to use

OPTIONS:
    -f, --format <FORMAT>    Output format [default: text] [possible values: text, json]
    -h, --help               Print help information
    -v, --verbose            Make output more verbose
    -V, --version            Print version information
```

## Using jfifdump as a library

To use jfifdump as a library add the following to your Cargo.toml dependencies:

```toml
jfifdump = "0.3"
```

## Example: Print image dimensions

```rust
use jfifdump::{Reader, Segment, JfifError};
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), JfifError> {
    let file = File::open("some.jpeg")?;

    let mut reader = Reader::new(BufReader::new(file))?;

    loop {
        match reader.next_segment()? {
            Segment::Eoi => break,
            Segment::Frame(frame) => {
                println!("{}x{}", frame.dimension_x, frame.dimension_y);
                break;
            }
            _ => {
                // Ignore other segments
            }
        }
    }

    Ok(())
}
```

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in jfifdump by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
