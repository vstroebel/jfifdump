use clap::{App, crate_name, crate_version, crate_description, crate_authors, Arg};
use std::fs::File;
use std::process::exit;
use std::io::BufReader;
use std::fmt::Write;
use jfifdump::*;
use json::{object, JsonValue};
use json::object::Object;

pub fn main() {
    let matches = create_clap_app().get_matches();

    let path = matches.value_of_os("INPUT").expect("Required arg present");

    let format = matches.value_of("FORMAT").unwrap_or("TEXT");

    let verbose = matches.is_present("VERBOSE");

    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to open file {}: {}", path.to_string_lossy(), err);
            exit(1);
        }
    };

    let res = match format {
        "json" => {
            let mut reader = JsonWriter::new(verbose);
            reader.read(file).map(|json| {
                println!("{}", json);
            })
        }
        _ => read_text(file),
    };

    if let Err(err) = res {
        eprintln!("Error reading file: {}", err);
        exit(1);
    }
}

fn create_clap_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(Arg::with_name("FORMAT")
            .short("f")
            .long("format")
            .possible_values(&["text", "json"])
            .default_value("text")
            .help("Output format"))
        .arg(Arg::with_name("VERBOSE")
            .short("v")
            .long("verbose")
            .empty_values(true)
            .help("Make output more verbose"))
        .arg(Arg::with_name("INPUT")
            .help("Jpeg file to use")
            .required(true))
}

fn read_text(file: File) -> Result<(), JfifError> {
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

fn get_marker_string(data: &[u8], max: usize) -> String {
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

fn get_sof_name(marker: u8) -> &'static str {
    match marker {
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

fn handle_frame(frame: &Frame) {
    let sof = get_sof_name(frame.sof);

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

struct JsonWriter {
    markers: Vec<JsonValue>,
    verbose: bool,
}

impl JsonWriter {
    pub fn new(verbose: bool) -> JsonWriter {
        JsonWriter {
            markers: vec![],
            verbose,
        }
    }

    pub fn read(&mut self, file: File) -> Result<String, JfifError> {
        let mut reader = Reader::new(BufReader::new(file))?;

        loop {
            match reader.next_segment()? {
                Segment::Eoi => break,
                Segment::App { nr, data } => self.handle_app(nr, &data),
                Segment::App0Jfif(jfif) => self.handle_app0_jfif(&jfif),
                Segment::Dqt(tables) => self.handle_dqt(&tables),
                Segment::Dht(tables) => self.handle_dht(&tables),
                Segment::Dac(dac) => self.handle_dac(&dac),
                Segment::Frame(frame) => self.handle_frame(&frame),
                Segment::Scan(scan) => self.handle_scan(&scan),
                Segment::Dri(restart) => self.handle_dri(restart),
                Segment::Rst(rst) => self.handle_rst(&rst),
                Segment::Comment(data) => self.handle_comment(&data),
                Segment::Unknown { marker, data } => self.handle_unknown(marker, &data),
            };
        }

        Ok(self.stringify())
    }

    fn add(&mut self, value: Object) {
        self.markers.push(JsonValue::Object(value));
    }

    fn handle_app(&mut self, nr: u8, data: &[u8]) {
        let mut value = Object::new();
        value.insert("marker", format!("App(0x{:X})", nr).into());

        value.insert("start", get_marker_string(data, 20).into());

        if self.verbose {
            value.insert("data", data.into());
        }

        self.add(value);
    }

    fn handle_app0_jfif(&mut self, jfif: &App0Jfif) {
        let mut value = Object::new();
        value.insert("marker", "App(0x0):JFIF".into());

        let mut density = Object::new();

        match jfif.unit {
            0 => density.insert("unit", "pixel".into()),
            1 => density.insert("unit", "dpi".into()),
            2 => density.insert("unit", "dpcm".into()),
            _ => density.insert("unit", format!("unknown {}", jfif.unit).into()),
        };

        density.insert("x", jfif.x_density.into());
        density.insert("y", jfif.y_density.into());
        value.insert("density", density.into());

        let mut thumbnail = Object::new();
        thumbnail.insert("width", jfif.x_thumbnail.into());
        thumbnail.insert("height", jfif.y_thumbnail.into());

        if self.verbose {
            if let Some(data) = &jfif.thumbnail {
                thumbnail.insert("data", data.clone().into());
            }
        }

        value.insert("thumbnail", thumbnail.into());

        self.add(value);
    }

    fn handle_dqt(&mut self, tables: &[Dqt]) {
        let mut value = Object::new();
        value.insert("marker", "DQT".into());

        let tables: Vec<JsonValue> = tables.iter().map(|table| {
            let mut t_value = Object::new();
            t_value.insert("dest", table.dest.into());
            t_value.insert("precision", table.precision.into());

            if self.verbose {
                t_value.insert("data", table.values.to_vec().into());
            }

            JsonValue::Object(t_value)
        }).collect();

        value.insert("tables", tables.into());

        self.add(value);
    }

    fn handle_dht(&mut self, tables: &[Dht]) {
        let mut value = Object::new();
        value.insert("marker", "DHT".into());

        let tables: Vec<JsonValue> = tables.iter().map(|table| {
            let mut t_value = Object::new();
            t_value.insert("class", table.class.into());
            t_value.insert("dest", table.dest.into());

            if self.verbose {
                t_value.insert("code_lengths", table.code_lengths.to_vec().into());
                t_value.insert("values", table.values.to_vec().into());
            }

            JsonValue::Object(t_value)
        }).collect();

        value.insert("tables", tables.into());

        self.add(value);
    }

    fn handle_dac(&mut self, dac: &Dac) {
        let mut value = Object::new();
        value.insert("marker", "DAC".into());

        let params: Vec<JsonValue> = dac.params.iter().map(|param| {
            object! {
                class: param.class,
                dest: param.dest,
                param: param.value,
            }
        }).collect();

        value.insert("params", params.into());

        self.add(value);
    }

    fn handle_frame(&mut self, frame: &Frame) {
        let sof = get_sof_name(frame.sof);

        let mut value = Object::new();
        value.insert("marker", "SOF".into());
        value.insert("type", sof.into());

        value.insert("precision", frame.precision.into());
        value.insert("dimension", object! {
           width:  frame.dimension_x,
            height: frame.dimension_y,
        });

        value.insert("components", frame.components.iter().map(|component| {
            object! {
                id: component.id,
                sampling_facor: object! {
                    horizontal: component.horizontal_sampling_factor,
                    vertical: component.vertical_sampling_factor,
                },
                quantization_table: component.quantization_table,
            }
        }).collect::<Vec<_>>().into());

        self.add(value);
    }

    fn handle_scan(&mut self, scan: &Scan) {
        let mut value = Object::new();
        value.insert("marker", "SOS".into());

        value.insert("components", scan.components.iter().map(|component| {
            object! {
                id: component.id,
                dc_table: component.dc_table,
                ac_table: component.ac_table,
            }
        }).collect::<Vec<_>>().into());

        value.insert("selection", object! {
            start: scan.selection_start,
            end: scan.selection_end,
        });

        value.insert("approximation", object! {
            low: scan.approximation_low,
            high: scan.approximation_high,
        });

        value.insert("size", scan.data.len().into());

        if self.verbose {
            value.insert("data", scan.data.clone().into());
        }

        self.add(value);
    }

    fn handle_dri(&mut self, restart: u16) {
        let mut value = Object::new();
        value.insert("marker", "DRI".into());
        value.insert("restart", restart.into());

        self.add(value);
    }

    fn handle_rst(&mut self, restart: &Rst) {
        let mut value = Object::new();
        value.insert("marker", format!("RST({})", restart.nr).into());

        value.insert("size", restart.data.len().into());

        if self.verbose {
            value.insert("data", restart.data.clone().into());
        }

        self.add(value);
    }

    fn handle_comment(&mut self, data: &[u8]) {
        let mut value = Object::new();
        value.insert("marker", "COM".into());

        if let Ok(comment) = std::str::from_utf8(data) {
            value.insert("text", comment.into());
        } else {
            value.insert("raw", data.into());
        }

        self.add(value);
    }

    fn handle_unknown(&mut self, marker: u8, data: &[u8]) {
        let mut value = Object::new();
        value.insert("marker", format!("Marker(0x{:X})", marker).into());

        value.insert("size", data.len().into());

        if self.verbose {
            value.insert("data", data.into());
        }
        self.add(value);
    }

    pub fn stringify(&self) -> String {
        json::stringify_pretty(JsonValue::Array(self.markers.clone()), 4)
    }
}
