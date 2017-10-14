extern crate clap;

use clap::{Arg, App};

extern crate time;

use time::Duration;

extern crate termcolor;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use std::fs::File;
use std::io::Write;

extern crate pcre;
extern crate either;

mod interpreter;

use interpreter::ill::{Interpreter, IllError};

mod opcodes;

pub struct NamedFile {
    file: File,
    name: String,
}

fn main() {
    let arg_matches = App::new("ill interpreter")
        .version("0.1B")
        .author("haze booth <admin@haze.pw>")
        .about("the (pretty) ill tiny language interpreter")
        .arg(
            Arg::with_name("inputs")
                .help("the ill source files")
                .required(true)
                .multiple(true),
        )
        .arg(Arg::with_name("preamble").long("preamble").takes_value(true).short("pre").multiple(true).help("load these files before we execute the main ones."))
        .arg(Arg::with_name("debug").help("show debug text").short("d"))
        .get_matches();

    let input_files_str: Vec<_> = arg_matches.values_of("inputs").unwrap().collect();

    let preamble_files;
    if arg_matches.is_present("preamble") {
        let preamble_files_str: Vec<_> = arg_matches.values_of("preamble").unwrap().collect();
        preamble_files = preamble_files_str
            .iter()
            .filter(|x| File::open(x).is_ok())
            .map(|x| {
                NamedFile {
                    file: File::open(x).unwrap(),
                    name: String::from(*x)
                }
            }).collect();
    } else {
        preamble_files = Vec::new();
    }
    let input_files: Vec<_> = input_files_str
        .iter()
        .filter(|x| File::open(x).is_ok())
        .map(|x| {
            NamedFile {
                file: File::open(x).unwrap(),
                name: String::from(*x),
            }
        })
        .collect();

    let mut int: Interpreter = Interpreter::new(arg_matches.is_present("debug"), input_files, preamble_files, opcodes::ill::default_opcodes());
    let mut res: Option<IllError> = None;
    let dur = Duration::span(|| { res = int.begin_parsing(); });

    let mut out = StandardStream::stdout(ColorChoice::Always);

    if res.is_some() {
        let err = res.unwrap();
        out.set_color(ColorSpec::new().set_fg(Some(Color::Red)))
            .ok();
        write!(&mut out, "{}", err.name()).ok();
        out.set_color(ColorSpec::new().set_fg(Some(Color::White)))
            .ok();
        print!(": {}\n", err);
    }

    println!(
        "PILL Execution took: {}s, ({} ms)",
        dur.num_seconds(),
        dur.num_milliseconds()
    );
}
