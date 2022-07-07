use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum JfifError {
    JfifMarkerNotFound,
    InvalidMarker(u8),
    InvalidMarkerLength(usize),
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
