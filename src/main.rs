#![deny(warnings, missing_copy_implementations)]
extern crate glob;
extern crate libc;
extern crate rubyfmt;

use std::fs::{metadata, read_to_string, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::exit;

use glob::glob;

#[derive(Debug)]
enum FileError {
    IO(io::Error),
    SyntaxError,
}

fn rubyfmt_file(file_path: PathBuf) -> Result<(), FileError> {
    let buffer = read_to_string(file_path.clone()).map_err(FileError::IO)?;
    let res = rubyfmt::format_buffer(&buffer);
    match res {
        Ok(res) => {
            let mut file = OpenOptions::new()
                .write(true)
                .open(file_path)
                .expect("file");
            write!(file, "{}", res).map_err(FileError::IO)?;
            Ok(())
        }
        Err(rubyfmt::RichFormatError::SyntaxError) => Err(FileError::SyntaxError),
        Err(e) => handle_error_from(e, &format!("{}", file_path.display())),
    }
}

fn rubyfmt_dir(path: &str) -> io::Result<()> {
    for entry in glob(&format!("{}/**/*.rb", path)).expect("it exists") {
        let p = entry.expect("should not be null");
        let res = rubyfmt_file(p.clone());
        if let Err(FileError::SyntaxError) = res {
            eprintln!(
                "warning: {} contains syntax errors, ignoring for now",
                p.display()
            );
        }
    }
    Ok(())
}

fn format_parts(parts: &[String]) {
    for part in parts {
        if let Ok(md) = metadata(part) {
            if md.is_dir() {
                rubyfmt_dir(part).expect("failed to format dir");
            } else if md.is_file() {
                rubyfmt_file(part.into()).expect("failed to format file");
            }
        }
    }
}

fn handle_error_from(e: rubyfmt::RichFormatError, source: &str) -> ! {
    use rubyfmt::RichFormatError::*;
    match e {
        SyntaxError => {
            eprintln!("{} contained invalid ruby syntax", source);
            exit(1);
        }
        rubyfmt::RichFormatError::RipperParseFailure(_) => {
            let bug_report = "
🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛
🐛                                                                                              🐛
🐛  Rubyfmt failed to correctly deserialize a tree from ripper. This is absolutely a bug        🐛
🐛  and you should send us a bug report at https://github.com/penelopezone/rubyfmt/issues/new.  🐛
🐛  Ideally you would include the full source code of the program you ran rubyfmt with.         🐛
🐛  If you can't do that for some reason, the best thing you can do is                          🐛
🐛  rerun rubyfmt on this program with the debug binary with `2>log_file` on the end            🐛
🐛  and then send us the log file that gets generated.                                          🐛
🐛                                                                                              🐛
🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛🐛
            ";
            eprintln!("{}", bug_report);
            exit(1);
        }
        IOError(e) => {
            eprintln!("IO error occured while running rubyfmt: {:?}, this may indicate a programming error, please file a bug report at https://github.com/penelopezone/rubyfmt/issues/new", e);
            exit(1)
        }
        rubyfmt::RichFormatError::OtherRubyError(s) => {
            eprintln!("A ruby error occured: {}, please file a bug report at https://github.com/penelopezone/rubyfmt/issues/new", s);
            exit(1)
        }
    }
}

fn main() {
    let res = rubyfmt::rubyfmt_init();
    if res != rubyfmt::InitStatus::OK as libc::c_int {
        panic!("bad init status");
    }
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        // consume stdin
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("reading frmo stdin to not fail");
        let res = rubyfmt::format_buffer(&buffer);
        match res {
            Ok(res) => {
                write!(io::stdout(), "{}", res).expect("write works");
                io::stdout().flush().expect("flush works");
            }
            Err(e) => handle_error_from(e, "stdin"),
        }
    } else if args.len() == 2 {
        // consume a filename
        if let Ok(md) = metadata(args[1].clone()) {
            if md.is_dir() {
                format_parts(&[args[1].clone()])
            } else {
                let buffer = read_to_string(args[1].clone()).expect("file exists");
                let res = rubyfmt::format_buffer(&buffer);
                match res {
                    Ok(res) => {
                        write!(io::stdout(), "{}", res).expect("write works");
                        io::stdout().flush().expect("flush works");
                    }
                    Err(e) => handle_error_from(e, &args[1]),
                }
            }
        } else {
            eprintln!("{} does not exist", args[1]);
            exit(1)
        }
    } else if args[1] == "-i" {
        // inline a file or directory
        let parts = &args[2..args.len()];
        format_parts(parts);
    } else {
        // inline many files and directories
        let parts = &args[1..args.len()];
        format_parts(parts);
    }
}
