#![allow(clippy::uninlined_format_args)]

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::exit;

use clap::{crate_description, crate_name, crate_version, Arg, Command, value_parser, ArgAction};

use jfifdump::*;

pub fn main() {
    let matches = create_clap_app().get_matches();

    let path = matches.get_one::<PathBuf>("INPUT").expect("Required arg present");

    let format = matches.get_one::<String>("FORMAT").map(|s|s.as_str()).unwrap_or("text");

    let verbose = matches.contains_id("VERBOSE");

    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to open file {}: {}", path.to_string_lossy(), err);
            exit(1);
        }
    };

    let bufread = BufReader::new(file);

    let res = match format {
        "json" => {
            let mut handler = JsonFormat::new(verbose);

            read(bufread, &mut handler).map(|_| {
                println!("{}", handler.stringify());
            })
        }
        _ => {
            let mut handler = TextFormat::new(verbose);
            read(bufread, &mut handler)
        }
    };

    if let Err(err) = res {
        eprintln!("Error reading file: {}", err);
        exit(1);
    }
}

fn create_clap_app() -> Command {
    Command::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::new("FORMAT")
                .short('f')
                .long("format")
                .value_parser(["text", "json"])
                .default_value("text")
                .help("Output format"),
        )
        .arg(
            Arg::new("VERBOSE")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Make output more verbose"),
        )
        .arg(
            Arg::new("INPUT")
                .help("Jpeg file to use")
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
}
