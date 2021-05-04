# JFIF-Dump

Dump structure of a jpeg file

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
USAGE:
    jfifdump [FLAGS] [OPTIONS] <INPUT>

FLAGS:
    -v, --verbose    Make output more verbose
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --format <FORMAT>    Output format [default: text]  [possible values: text, json]

ARGS:
    <INPUT>    Jpeg file to use

```

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in serde_urlencoded by you, as defined in the Apache-2.0 license, 
shall be dual licensed as above, without any additional terms or conditions.
