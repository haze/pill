extern crate clap;
extern crate time;
extern crate termcolor;
extern crate pcre;
extern crate either;
extern crate gag;

use clap::{Arg, App};
use time::Duration;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::fs::File;
use std::io::Write;
mod interpreter;
use interpreter::ill::{Interpreter, AdvancedIllError};
mod opcodes;



pub struct NamedFile {
    file: File,
    name: String,
}

fn repeat(times: i32, char: char) -> String {
    let mut buff: String = String::new();
    for _ in 0..times {
        buff.push(char);
    }
    buff
}


fn main() {
    let arg_matches = App::new("ill interpreter")
        .version("0.8F")
        .author("haze booth <admin@haze.pw>")
        .about("the (pretty) ill tiny language interpreter")
        .arg(
            Arg::with_name("inputs")
                .help("the ill source files")
                .required(true)
                .multiple(true),
        )
        .arg(Arg::with_name("preamble").long("preamble").takes_value(true).short("pre").multiple(true).help("load these files before we execute the main ones."))
        .arg(Arg::with_name("debug").help("show debug text").short("d").long("debug"))
        .arg(Arg::with_name("quiet").help("only show program output").short("q").long("quiet"))
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
    let mut int: Interpreter = Interpreter::new(arg_matches.is_present("debug"), arg_matches.is_present("quiet"), input_files, preamble_files, opcodes::ill::default_opcodes());
    let mut res: Option<AdvancedIllError> = None;
    let dur = Duration::span(|| { res = int.begin_parsing(); });
    let mut out = StandardStream::stdout(ColorChoice::Always);

    if res.is_some() {
        let err = res.unwrap();
        out.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
            .ok();
        let err_head_line = if err.head.is_some() { err.head.unwrap().line } else { -1 };
        let error_str = err.get_error_portion();
        if error_str.is_some() {
            let xstr = error_str.unwrap();
            let head = err.head.unwrap();
            let space_push_buffer = repeat(head.line.to_string().len() as i32, ' ');
            writeln!(&mut out, "    {}{}", space_push_buffer, err.error.name()).ok();
            print!("{}--> ", space_push_buffer);
            out.set_color(ColorSpec::new().set_fg(Some(Color::White)))
                .ok();
            println!("{}:{}:{}", err.file.filename, head.line, head.column);
            out.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
                .ok();
            for line in (err_head_line - 1)..(err_head_line + 2) {
                out.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))
                    .ok();
                if line == err_head_line {
                    print!("{} |", line);
                    out.set_color(ColorSpec::new().set_fg(Some(Color::White)))
                        .ok();
                    println!(" {}", xstr);
                } else if line == (err_head_line + 1) {
                    let err_pointer_buffer = repeat(head.column - 1, ' ');
                    print!("{} |{}", line, err_pointer_buffer);
                    let err_tail = repeat((xstr.len() as i32 - head.column), '-');
                    print!(" ^{}", err_tail);
                    out.set_color(ColorSpec::new().set_fg(Some(Color::White)))
                        .ok();
                    println!(" {}", err.error.get_actual_desc());
                } else {
                    println!("{} |", line);
                }
            }
        }
        out.set_color(ColorSpec::new().set_fg(Some(Color::White)))
            .ok();
    }

    if !int.quiet {
        println!(
            "PILL Execution took: {}s, ({} ms)",
            dur.num_seconds(),
            dur.num_milliseconds()
        );
    }
}