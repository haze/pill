extern crate clap;
use clap::{Arg, App};

extern crate time;
use time::Duration;

use std::fs::File;

mod interpreter;
use interpreter::ill::{Interpreter, IllError};

fn main() {
    let arg_matches = App::new("ill interpreter")
        .version("0.1")
        .author("haze booth <admin@haze.pw>")
        .about("the (pretty) ill tiny language interpreter")
        .arg(Arg::with_name("inputs")
            .help("the ill source files")
            .required(true)
            .multiple(true))
        .arg(Arg::with_name("debug")
            .help("show debug text")
            .short("d"))
        .get_matches();
    let input_files_str: Vec<_> = arg_matches.values_of("inputs").unwrap().collect();
    let input_files: Vec<File> = input_files_str.iter()
        .filter(|fin| File::open(fin).is_ok())
        .map(|x| File::open(x).unwrap()).collect();
    
    let mut int: Interpreter = Interpreter::new(arg_matches.is_present("debug"), input_files);
    let mut res: Result<(), IllError> = Ok(());
    let dur = Duration::span(||
        {
            res = int.begin_parsing();
        }
    );
    if res.is_err() {
        println!("[ERROR]: {}", res.err().unwrap());
    }
    println!("PILL Execution took: {}s, ({} ms)", dur.num_seconds(), dur.num_milliseconds());
}
