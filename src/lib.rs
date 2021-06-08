//! # JFIF Dump
//!
//! A crate for reading the content of a JFIF file without decoding JPEG image data.
//!
//! ## Example: Print image dimensions
//!
//! ```no_run
//! # use jfifdump::JfifError;
//! # fn main() -> Result<(), JfifError> {
//!
//! use jfifdump::{Reader, Segment};
//! use std::fs::File;
//! use std::io::BufReader;
//!
//! let file = File::open("some.jpeg")?;
//!
//! let mut reader = Reader::new(BufReader::new(file))?;
//!
//! loop {
//!     match reader.next_segment()? {
//!         Segment::Eoi => break,
//!         Segment::Frame(frame) => {
//!             println!("{}x{}", frame.dimension_x, frame.dimension_y);
//!             break;
//!         }
//!         _ => {
//!             // Ignore other segments
//!         }
//!     }
//! }
//!
//! # Ok(())
//! }
//! ```

mod error;
mod reader;
mod handler;
mod json;
mod text;

pub use error::JfifError;
pub use reader::{Reader, Segment, App0Jfif, Frame, FrameComponent, Scan, ScanComponent, Dht, Dqt, Dac, Rst};
pub use handler::Handler;
pub use text::TextFormat;
pub use crate::json::JsonFormat;

use std::io::Read;

/// Read JFIF input and call handler for all segments
pub fn read<H: Handler, R: Read>(input: R, handler: &mut H) -> Result<(), JfifError> {
    let mut reader = Reader::new(input)?;

    loop {
        match reader.next_segment()? {
            Segment::Eoi => break,
            Segment::App { nr, data } => handler.handle_app(nr, &data),
            Segment::App0Jfif(jfif) => handler.handle_app0_jfif(&jfif),
            Segment::Dqt(tables) => handler.handle_dqt(&tables),
            Segment::Dht(tables) => handler.handle_dht(&tables),
            Segment::Dac(dac) => handler.handle_dac(&dac),
            Segment::Frame(frame) => handler.handle_frame(&frame),
            Segment::Scan(scan) => handler.handle_scan(&scan),
            Segment::Dri(restart) => handler.handle_dri(restart),
            Segment::Rst(rst) => handler.handle_rst(&rst),
            Segment::Comment(data) => handler.handle_comment(&data),
            Segment::Unknown { marker, data } => handler.handle_unknown(marker, &data),
        };
    }

    Ok(())
}
