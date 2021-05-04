mod error;
mod reader;
mod handler;
mod json;

pub use error::JfifError;
pub use reader::{Reader, Segment, App0Jfif, Frame, FrameComponent, Scan, ScanComponent, Dht, Dqt, Dac, Rst};
pub use handler::Handler;
pub use crate::json::JsonFormat;

use std::fs::File;
use std::io::BufReader;

pub fn read<H: Handler>(file: File, handler: &mut H) -> Result<(), JfifError> {
    let mut reader = Reader::new(BufReader::new(file))?;

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