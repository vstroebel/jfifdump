use clap::{App, crate_name, crate_version, crate_description, crate_authors, Arg};
use std::fs::File;
use std::process::exit;
use std::io::BufReader;
use jfifdump::*;

pub fn main() {
    let matches = create_clap_app().get_matches();

    let path = matches.value_of_os("INPUT").expect("Required arg present");

    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to open file {}: {}", path.to_string_lossy(), err);
            exit(1);
        }
    };

    if let Err(err) = read(file) {
        eprintln!("Error reading file: {}", err);
        exit(1);
    } else {
        println!("Have a nice day");
    }
}

fn create_clap_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(Arg::with_name("INPUT")
            .help("Jpeg file to use")
            .required(true))
}

fn read(file: File) -> Result<(), JfifError> {
    let mut reader = Reader::new(BufReader::new(file))?;

    loop {
        match reader.next_segment()? {
            Segment::Eoi => break,
            Segment::App { nr, data } => handle_app(nr, &data),
            Segment::App0Jfif(jfif) => handle_app0_jfif(&jfif),
            Segment::Dqt(tables) => handle_dqt(&tables),
            Segment::Dht(tables) => handle_dht(&tables),
            Segment::Dac(dac) => handle_dac(&dac),
            Segment::Frame(frame) => handle_frame(&frame),
            Segment::Scan(scan) => handle_scan(&scan),
            Segment::Dri(restart) => handle_dri(restart),
            Segment::Rst(rst) => handle_rst(&rst),
            Segment::Comment(data) => handle_comment(&data),
            Segment::Unknown { marker, data } => handle_unknown(marker, &data),
        }
    }

    Ok(())
}

fn print_ascii_value(v: u8) {
    if v.is_ascii_graphic() || v == 0x20 {
        print!("{}", v as char);
    } else {
        print!("\\x{:#04X}", v);
    }
}

fn handle_app(nr: u8, data: &[u8]) {
    print!("App(0x{:X}):", nr);

    for &v in data.iter().take(20) {
        print_ascii_value(v);
    }

    println!();
}

fn handle_app0_jfif(jfif: &App0Jfif) {
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

fn handle_dqt(tables: &[Dqt]) {
    println!("DQT:");

    for table in tables {
        print!("  {}: Precision {}", table.dest, table.precision);
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
        println!();
    }
}

fn handle_dht(tables: &[Dht]) {
    println!("DHT:");

    for table in tables {
        print!("  Table {}: Class {}\n    Code lengths: ", table.dest, table.class);
        for (i, &v) in table.code_lengths.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", v)
        }
        println!();
    }
}

fn handle_dac(dac: &Dac) {
    println!("DAC:");

    for param in &dac.params {
        println!("  Class: {}   Dest: {}    Value: {}", param.class, param.dest, param.value);
    }
}

fn handle_frame(frame: &Frame) {
    let sof = match frame.sof {
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
    };

    println!("Frame: {}", sof);
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

fn handle_scan(scan: &Scan) {
    println!("Scan: ");

    for component in &scan.components {
        println!("  Component: {} DC:{} AC:{}", component.id, component.dc_table, component.dc_table);
    }

    println!("  Selection: {} to {}", scan.selection_start, scan.selection_end);
    println!("  Approximation: {} to {}", scan.approximation_low, scan.approximation_high);
    println!("  Data: {} bytes", scan.data.len());
}

fn handle_dri(restart: u16) {
    println!("DRI: {}", restart);
}

fn handle_rst(restart: &Rst) {
    println!("RST({}): Data: {} bytes", restart.nr, restart.data.len());
}

fn handle_comment(data: &[u8]) {
    if let Ok(comment) = std::str::from_utf8(data) {
        println!("Comment: {}", comment);
    } else {
        println!("Comment: BAD STRING WITH LENGTH {}", data.len());
    }
}

fn handle_unknown(marker: u8, data: &[u8]) {
    println!("Unknown(0x{:X}):{}", marker, data.len());
}
