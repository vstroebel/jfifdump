use std::io::{Read, Error as IoError};
use std::fmt::Write;

pub use crate::JfifError;

pub struct Reader<R: Read> {
    reader: R,
    current_marker: Option<u8>,
}

impl<R: Read> Reader<R> {
    pub fn new(mut reader: R) -> Result<Self, JfifError> {
        let mut buf = [0u8; 2];

        if reader.read(&mut buf)? != 2 || buf != [0xFF, 0xD8] {
            return Err(JfifError::JfifMarkerNotFound);
        }

        Ok(Self {
            reader,
            current_marker: None,
        })
    }

    fn read_u8(&mut self) -> Result<u8, IoError> {
        let mut buf = [0u8];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u4_tuple(&mut self) -> Result<(u8, u8), IoError> {
        let v = self.read_u8()?;
        Ok((v >> 4, v & 0x0F))
    }

    fn read_u16(&mut self) -> Result<u16, IoError> {
        let mut buf = [0u8; 2];
        self.reader.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_to_vec(&mut self, length: usize) -> Result<Vec<u8>, IoError> {
        let mut result = vec![0u8; length];
        self.reader.read_exact(&mut result)?;
        Ok(result)
    }

    fn skip(&mut self, length: usize) -> Result<(), IoError> {
        for _ in 0..length {
            self.read_u8()?;
        }
        Ok(())
    }

    fn read_length(&mut self) -> Result<usize, JfifError> {
        let length = self.read_u16()? as usize;

        if length <= 2 {
            return Err(JfifError::InvalidMarkerLength(length));
        }

        Ok(length - 2)
    }

    pub fn next_segment(&mut self) -> Result<Segment, JfifError> {
        let marker = if let Some(marker) = self.current_marker.take() {
            marker
        } else {
            while self.read_u8()? != 0xFF {}

            let mut byte = self.read_u8()?;

            while byte == 0xFF {
                byte = self.read_u8()?;
            }
            byte
        };

        match marker {
            0x00 => Err(JfifError::InvalidMarker(0x00)),
            0xD9 => Ok(Segment::Eoi),
            0xE0..=0xEF => Ok(self.read_app_segment(marker - 0xE0)?),
            0xDB => Ok(Segment::Dqt(self.read_dqt()?)),
            0xC4 => Ok(Segment::Dht(self.read_dht()?)),
            0xCC => Ok(Segment::Dac(self.read_dac()?)),
            0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF => Ok(Segment::Frame(self.read_frame(marker)?)),
            0xDA => Ok(Segment::Scan(self.read_scan()?)),
            0xDD => Ok(Segment::Dri(self.read_dri()?)),
            0xD0..=0xD7 => Ok(Segment::Rst(self.read_rst(marker - 0xD0)?)),
            0xFE => Ok(Segment::Comment(self.read_segment()?)),
            marker => Ok(Segment::Unknown {
                marker,
                data: self.read_segment()?,
            }),
        }
    }

    fn read_segment(&mut self) -> Result<Vec<u8>, JfifError> {
        let length = self.read_length()?;
        Ok(self.read_to_vec(length)?)
    }

    fn read_app_segment(&mut self, nr: u8) -> Result<Segment, JfifError> {
        let data = self.read_segment()?;

        if nr == 0 && data.len() >= 14 && data.starts_with(b"JFIF\0") {
            let major = data[5];
            let minor = data[6];

            let unit = data[7];
            let x_density = u16::from_be_bytes([data[8], data[9]]);
            let y_density = u16::from_be_bytes([data[10], data[11]]);

            let x_thumbnail = data[12];
            let y_thumbnail = data[13];

            let thumbnail = if x_thumbnail > 0 && y_thumbnail > 0 && data.len() > 14 {
                Some(data[14..].to_vec())
            } else {
                None
            };

            return Ok(Segment::App0Jfif(App0Jfif {
                major,
                minor,
                unit,
                x_density,
                y_density,
                x_thumbnail,
                y_thumbnail,
                thumbnail,
            }));
        }


        Ok(Segment::App {
            nr,
            data,
        })
    }

    fn read_dqt(&mut self) -> Result<Vec<Dqt>, JfifError> {
        let length = self.read_length()?;

        let num_tables = length / 65;

        let mut tables = vec![];

        for _ in 0..num_tables {
            let (precision, dest) = self.read_u4_tuple()?;

            let mut values = [0u8; 64];
            self.reader.read_exact(&mut values)?;

            tables.push(Dqt {
                precision,
                dest,
                values: Box::new(values),
            });
        }

        let remaining = length - num_tables * 65;
        if remaining > 0 {
            self.skip(remaining)?;
        }

        Ok(tables)
    }

    fn read_dht(&mut self) -> Result<Vec<Dht>, JfifError> {
        let mut length = self.read_length()?;

        let mut tables = vec![];

        while length > 17 {
            let (class, destination) = self.read_u4_tuple()?;
            let mut code_lengths = [0u8; 16];
            self.reader.read_exact(&mut code_lengths)?;

            let num_codes = code_lengths.iter().map(|v| *v as usize).sum();

            let values = self.read_to_vec(num_codes)?;

            tables.push(Dht {
                class,
                dest: destination,
                code_lengths,
                values,
            });

            length -= 17 + num_codes;
        }

        if length > 0 {
            self.skip(length)?;
        }

        Ok(tables)
    }

    fn read_dac(&mut self) -> Result<Dac, JfifError> {
        let length = self.read_length()?;

        let mut params = vec![];

        for _ in 0..(length / 2) {
            let (class, dest) = self.read_u4_tuple()?;
            let value = self.read_u8()?;

            params.push(DacParam {
                class,
                dest,
                value,
            })
        }

        Ok(Dac {
            params,
        })
    }

    fn read_scan(&mut self) -> Result<Scan, JfifError> {
        let length = self.read_length()?;
        let num_components = self.read_u8()?;

        let mut components = vec![];

        for _ in 0..num_components {
            let id = self.read_u8()?;
            let (dc_table, ac_table) = self.read_u4_tuple()?;

            components.push(ScanComponent {
                id,
                dc_table,
                ac_table,
            })
        };

        let selection_start = self.read_u8()?;
        let selection_end = self.read_u8()?;
        let (approximation_low, approximation_high) = self.read_u4_tuple()?;

        let remaining = length - 1 - num_components as usize * 2 - 3;

        if remaining > 0 {
            self.skip(remaining)?;
        }

        let data = self.read_scan_data()?;

        Ok(Scan {
            components,
            selection_start,
            selection_end,
            approximation_low,
            approximation_high,
            data,
        })
    }

    fn read_scan_data(&mut self) -> Result<Vec<u8>, JfifError> {
        let mut data = vec![];

        loop {
            let byte = self.read_u8()?;
            if byte == 0xFF {
                let byte = self.read_u8()?;
                if byte != 0x00 {
                    self.current_marker = Some(byte);
                    break;
                } else {
                    data.push(0xFF);
                    data.push(byte);
                }
            } else {
                data.push(byte);
            }
        }
        Ok(data)
    }

    fn read_rst(&mut self, nr: u8) -> Result<Rst, JfifError> {
        let data = self.read_scan_data()?;
        Ok(Rst {
            nr,
            data,
        })
    }

    fn read_dri(&mut self) -> Result<u16, JfifError> {
        let length = self.read_length()?;
        let restart = self.read_u16()?;

        let remaining = length - 2;

        if remaining > 0 {
            self.skip(remaining)?;
        }

        Ok(restart)
    }

    fn read_frame(&mut self, sof: u8) -> Result<Frame, JfifError> {
        let length = self.read_length()?;

        let precision = self.read_u8()?;
        let dimension_y = self.read_u16()?;
        let dimension_x = self.read_u16()?;

        let num_components = self.read_u8()?;

        let mut components = vec![];

        for _ in 0..num_components {
            let id = self.read_u8()?;
            let (horizontal_sampling_factor, vertical_sampling_factor) = self.read_u4_tuple()?;
            let quantization_table = self.read_u8()?;

            components.push(FrameComponent {
                id,
                horizontal_sampling_factor,
                vertical_sampling_factor,
                quantization_table,
            })
        }

        let remaining = length - 6 - num_components as usize * 3;

        if remaining > 0 {
            self.skip(remaining)?;
        }

        Ok(Frame {
            sof,
            precision,
            dimension_y,
            dimension_x,
            components,
        })
    }
}

pub enum Segment {
    Eoi,
    App {
        nr: u8,
        data: Vec<u8>,
    },
    App0Jfif(App0Jfif),
    Dqt(Vec<Dqt>),
    Dht(Vec<Dht>),
    Dac(Dac),
    Frame(Frame),
    Scan(Scan),
    Dri(u16),
    Rst(Rst),
    Comment(Vec<u8>),
    Unknown {
        marker: u8,
        data: Vec<u8>,
    },
}

#[derive(Debug)]
pub struct App0Jfif {
    pub major: u8,
    pub minor: u8,
    pub unit: u8,
    pub x_density: u16,
    pub y_density: u16,
    pub x_thumbnail: u8,
    pub y_thumbnail: u8,
    pub thumbnail: Option<Vec<u8>>,
}

pub struct Dqt {
    pub precision: u8,
    pub dest: u8,
    pub values: Box<[u8; 64]>,
}

pub struct Dht {
    pub class: u8,
    pub dest: u8,
    pub code_lengths: [u8; 16],
    pub values: Vec<u8>,
}

pub struct DacParam {
    pub class: u8,
    pub dest: u8,
    pub value: u8,
}

pub struct Dac {
    pub params: Vec<DacParam>,
}

pub struct ScanComponent {
    pub id: u8,
    pub dc_table: u8,
    pub ac_table: u8,
}

pub struct Scan {
    pub components: Vec<ScanComponent>,
    pub selection_start: u8,
    pub selection_end: u8,
    pub approximation_low: u8,
    pub approximation_high: u8,
    pub data: Vec<u8>,
}

pub struct Rst {
    pub nr: u8,
    pub data: Vec<u8>,
}

pub struct FrameComponent {
    pub id: u8,
    pub horizontal_sampling_factor: u8,
    pub vertical_sampling_factor: u8,
    pub quantization_table: u8,
}

pub struct Frame {
    pub sof: u8,
    pub precision: u8,
    pub dimension_y: u16,
    pub dimension_x: u16,
    pub components: Vec<FrameComponent>,
}

impl Frame {
    pub fn get_sof_name(&self) -> &'static str {
        match self.sof {
            0xC0 => "Baseline DCT",
            0xC1 => "Extended sequential DCT",
            0xC2 => "Progressive DCT",
            0xC3 => "Lossless",
            0xC5 => "Differential sequential DCT",
            0xC6 => "Differential progressiveDCT",
            0xC7 => "Differential lossless",
            0xC9 => "Extended sequential DCT arithmetic",
            0xCA => "Progressive DCT arithmetic",
            0xCB => "Lossless arithmetic coding",
            0xCD => "Differential sequential DCT arithmetic",
            0xCE => "Differential progressive DCT arithmetic",
            0xCF => "Differential lossless arithmetic",
            _ => "Unknown"
        }
    }
}

pub fn get_marker_string(data: &[u8], max: usize) -> String {
    let mut result = "".to_owned();
    for &v in data.iter().take(max) {
        if v.is_ascii_graphic() || v == 0x20 {
            result.push(v as char);
        } else {
            write!(result, "\\x{:#04X}", v).unwrap();
        }
    }

    result
}