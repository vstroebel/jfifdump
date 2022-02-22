use crate::{Handler, Rst, Scan, Frame, Dac, Dht, Dqt, App0Jfif};

pub struct TextFormat {
    verbose: bool,
}

impl TextFormat {
    pub fn new(verbose: bool) -> TextFormat {
        TextFormat {
            verbose
        }
    }
}

fn print_ascii_value(v: u8) {
    if v.is_ascii_graphic() || v == 0x20 {
        print!("{}", v as char);
    } else {
        print!("\\x{:#04X}", v);
    }
}

impl Handler for TextFormat {
    fn handle_app(&mut self, position: usize, nr: u8, data: &[u8]) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        print!("App(0x{:X}):", nr);

        for &v in data.iter().take(20) {
            print_ascii_value(v);
        }

        println!();
    }

    fn handle_app0_jfif(&mut self, position: usize, jfif: &App0Jfif) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        println!("App(0x0): JFIF");

        println!("  Version: {}.{:02}", jfif.major, jfif.minor);

        let unit = match jfif.unit {
            0 => "pixel".to_owned(),
            1 => "dots per inch".to_owned(),
            2 => "dots per cm".to_owned(),
            _ => format!("Unknown unit: {}", jfif.unit),
        };

        println!("  Density: {}x{} {}", jfif.x_density, jfif.y_density, unit);
        println!("  Thumbnail: {}x{}", jfif.x_thumbnail, jfif.y_thumbnail);
    }

    fn handle_dqt(&mut self, position: usize, tables: &[Dqt]) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        println!("DQT:");

        for table in tables {
            print!("  {}: Precision {}", table.dest, table.precision);
            if self.verbose {
                for (i, &v) in table.values.iter().enumerate() {
                    if i % 8 == 0 {
                        print!("\n    ");
                    }
                    if v < 10 {
                        print!(" ");
                    }
                    if v < 100 {
                        print!(" ");
                    }
                    print!("{}, ", v)
                }
            }
            println!();
        }
    }

    fn handle_dht(&mut self, position: usize, tables: &[Dht]) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        println!("DHT:");

        for table in tables {
            println!("  Table {}: Class {}", table.dest, table.class);
            if self.verbose {
                print!("    Code lengths: ");
                for (i, &v) in table.code_lengths.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    print!("{}", v)
                }
                println!();
            }
        }
    }

    fn handle_dac(&mut self, position: usize, dac: &Dac) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        println!("DAC:");

        for param in &dac.params {
            println!("  Class: {}   Dest: {}    Value: {}", param.class, param.dest, param.value);
        }
    }

    fn handle_frame(&mut self, position: usize, frame: &Frame) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        println!("Frame: {}", frame.get_sof_name());
        println!("  Precision: {}", frame.precision);
        println!("  Dimension: {}x{}", frame.dimension_x, frame.dimension_y);

        for component in &frame.components {
            println!("  Component({}): Sampling {}x{} Quantization: {}",
                     component.id,
                     component.horizontal_sampling_factor,
                     component.vertical_sampling_factor,
                     component.quantization_table);
        }
    }

    fn handle_scan(&mut self, position: usize, scan: &Scan) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        println!("Scan: ");

        for component in &scan.components {
            println!("  Component: {} DC:{} AC:{}", component.id, component.dc_table, component.dc_table);
        }

        println!("  Selection: {} to {}", scan.selection_start, scan.selection_end);
        println!("  Approximation: {} to {}", scan.approximation_low, scan.approximation_high);
        println!("  Data: {} bytes", scan.data.len());
    }

    fn handle_dri(&mut self, position: usize, restart: u16) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        println!("DRI: {}", restart);
    }

    fn handle_rst(&mut self, position: usize, restart: &Rst) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        println!("RST({}): Data: {} bytes", restart.nr, restart.data.len());
    }

    fn handle_comment(&mut self, position: usize, data: &[u8]) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        if let Ok(comment) = std::str::from_utf8(data) {
            println!("Comment: {}", comment);
        } else {
            println!("Comment: BAD STRING WITH LENGTH {}", data.len());
        }
    }

    fn handle_unknown(&mut self, position: usize, marker: u8, data: &[u8]) {
        if self.verbose {
            print!("0x{:X}: ", position);
        }
        println!("Unknown(0x{:X}):{}", marker, data.len());
    }
}