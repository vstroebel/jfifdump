use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum JfifError {
    JfifMarkerNotFound,
    InvalidMarker(u8),
    InvalidMarkerLength(usize),
    InvalidDhtSegmentLength(usize),
    InvalidDqtSegmentLength(usize),
    InvalidFrameSegmentLength(usize),
    InvalidDriLength(usize),
    InvalidScanHeaderLength(usize),
    IoError(std::io::Error),
}

impl From<std::io::Error> for JfifError {
    fn from(err: std::io::Error) -> JfifError {
        JfifError::IoError(err)
    }
}

impl Display for JfifError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use JfifError::*;
        match self {
            JfifMarkerNotFound => write!(f, "Not a JFIF file"),
            InvalidMarker(value) => write!(f, "Invalid marker: 0x{:X}", value),
            InvalidMarkerLength(length) => write!(f, "Invalid length for marker: {}", length),
            InvalidDhtSegmentLength(length) => write!(f, "Invalid dht segment length: {}", length),
            InvalidDqtSegmentLength(length) => write!(f, "Invalid dqt segment length: {}", length),
            InvalidFrameSegmentLength(length) => write!(f, "Invalid dqt segment length: {}", length),
            InvalidDriLength(length) => write!(f, "Invalid dri length: {}", length),
            InvalidScanHeaderLength(length) => write!(f, "Invalid scan header length: {}", length),
            IoError(err) => err.fmt(f),
        }
    }
}

impl Error for JfifError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            JfifError::IoError(err) => Some(err),
            _ => None,
        }
    }
}
