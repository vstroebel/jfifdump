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
//! use jfifdump::{Reader, SegmentKind};
//! use std::fs::File;
//! use std::io::BufReader;
//!
//! let file = File::open("some.jpeg")?;
//!
//! let mut reader = Reader::new(BufReader::new(file))?;
//!
//! loop {
//!     match reader.next_segment()?.kind {
//!         SegmentKind::Eoi => break,
//!         SegmentKind::Frame(frame) => {
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

#![allow(clippy::uninlined_format_args)]

use std::io::{ErrorKind, Read};

pub use error::JfifError;
pub use handler::Handler;
pub use reader::{
    App0Jfif, Dac, Dht, Dqt, Frame, FrameComponent, Reader, Rst, Scan, ScanComponent, Segment,
    SegmentKind,
};
pub use text::TextFormat;

#[cfg(feature = "json")]
pub use crate::json::JsonFormat;

mod error;
mod handler;
#[cfg(feature = "json")]
mod json;
mod reader;
mod text;

/// Read JFIF input and call handler for all segments
pub fn read<H: Handler, R: Read>(input: R, handler: &mut H) -> Result<(), JfifError> {
    let mut reader = Reader::new(input)?;

    loop {
        let segment = match reader.next_segment() {
            Ok(segment) => segment,
            Err(JfifError::IoError(ioerror)) => {
                return if ioerror.kind() == ErrorKind::UnexpectedEof {
                    Ok(())
                } else {
                    Err(JfifError::IoError(ioerror))
                }
            }
            Err(err) => return Err(err),
        };

        match segment.kind {
            SegmentKind::Soi => handler.handle_soi(segment.position, segment.length),
            SegmentKind::Eoi => handler.handle_eoi(segment.position, segment.length),
            SegmentKind::App { nr, data } => {
                handler.handle_app(segment.position, segment.length, nr, &data)
            }
            SegmentKind::App0Jfif(jfif) => {
                handler.handle_app0_jfif(segment.position, segment.length, &jfif)
            }
            SegmentKind::Dqt(tables) => {
                handler.handle_dqt(segment.position, segment.length, &tables)
            }
            SegmentKind::Dht(tables) => {
                handler.handle_dht(segment.position, segment.length, &tables)
            }
            SegmentKind::Dac(dac) => handler.handle_dac(segment.position, segment.length, &dac),
            SegmentKind::Frame(frame) => {
                handler.handle_frame(segment.position, segment.length, &frame)
            }
            SegmentKind::Scan(scan) => handler.handle_scan(segment.position, segment.length, &scan),
            SegmentKind::Dri(restart) => {
                handler.handle_dri(segment.position, segment.length, restart)
            }
            SegmentKind::Rst(rst) => handler.handle_rst(segment.position, segment.length, &rst),
            SegmentKind::Comment(data) => {
                handler.handle_comment(segment.position, segment.length, &data)
            }
            SegmentKind::Unknown { marker, data } => {
                handler.handle_unknown(segment.position, segment.length, marker, &data)
            }
        };
    }
}
