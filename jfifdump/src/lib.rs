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

use std::io::Read;

pub use error::JfifError;
pub use handler::Handler;
pub use reader::{
    App0Jfif, Dac, Dht, Dqt, Frame, FrameComponent, Reader, Rst, Scan, ScanComponent, Segment,
    SegmentKind,
};
pub use text::TextFormat;

pub use crate::json::JsonFormat;

mod error;
mod handler;
mod json;
mod reader;
mod text;

/// Read JFIF input and call handler for all segments
pub fn read<H: Handler, R: Read>(input: R, handler: &mut H) -> Result<(), JfifError> {
    let mut reader = Reader::new(input)?;

    loop {
        let segment = reader.next_segment()?;
        match segment.kind {
            SegmentKind::Eoi => break,
            SegmentKind::App { nr, data } => handler.handle_app(segment.position, nr, &data),
            SegmentKind::App0Jfif(jfif) => handler.handle_app0_jfif(segment.position, &jfif),
            SegmentKind::Dqt(tables) => handler.handle_dqt(segment.position, &tables),
            SegmentKind::Dht(tables) => handler.handle_dht(segment.position, &tables),
            SegmentKind::Dac(dac) => handler.handle_dac(segment.position, &dac),
            SegmentKind::Frame(frame) => handler.handle_frame(segment.position, &frame),
            SegmentKind::Scan(scan) => handler.handle_scan(segment.position, &scan),
            SegmentKind::Dri(restart) => handler.handle_dri(segment.position, restart),
            SegmentKind::Rst(rst) => handler.handle_rst(segment.position, &rst),
            SegmentKind::Comment(data) => handler.handle_comment(segment.position, &data),
            SegmentKind::Unknown { marker, data } => {
                handler.handle_unknown(segment.position, marker, &data)
            }
        };
    }

    Ok(())
}
