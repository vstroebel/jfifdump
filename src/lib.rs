mod error;
mod reader;

pub use error::JfifError;
pub use reader::{Reader, Segment, App0Jfif, Frame, FrameComponent, Scan, ScanComponent, Dht, Dqt, Dac, Rst};
