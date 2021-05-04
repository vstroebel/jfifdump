use clap::{App, crate_name, crate_version, crate_description, crate_authors, Arg};
use std::fs::File;
use std::process::exit;
use jfifdump::*;

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
            let mut handler = JsonFormat::new(verbose);

            read(file, &mut handler).map(|_| {
                println!("{}", handler.stringify());
            })
        }
        _ => {
            let mut handler = TextFormat::new(verbose);
            read(file, &mut handler)
        }
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
