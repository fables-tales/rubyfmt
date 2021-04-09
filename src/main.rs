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
    Io(io::Error),
    SyntaxError,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ErrorExit {
    NoExit,
    Exit,
}

fn rubyfmt_file(file_path: PathBuf) -> Result<(), FileError> {
    let buffer = read_to_string(file_path.clone()).map_err(FileError::Io)?;
    let res = rubyfmt::format_buffer(&buffer);
    match res {
        Ok(res) => {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(file_path)
                .expect("file");
            write!(file, "{}", res).map_err(FileError::Io)?;
            Ok(())
        }
        Err(rubyfmt::RichFormatError::SyntaxError) => Err(FileError::SyntaxError),
        Err(e) => {
            // we're in a formatting loop, so print, and OK
            handle_error_from(e, &format!("{}", file_path.display()), ErrorExit::NoExit);
            Ok(())
        }
    }
}

fn rubyfmt_dir(path: &str) {
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
}

fn format_parts(parts: &[String]) {
    for part in parts {
        if let Ok(md) = metadata(part) {
            if md.is_dir() {
                rubyfmt_dir(part);
            } else if md.is_file() {
                rubyfmt_file(part.into()).expect("failed to format file");
            }
        }
    }
}

fn handle_error_from(err: rubyfmt::RichFormatError, source: &str, error_exit: ErrorExit) {
    use rubyfmt::RichFormatError::*;
    let e = || {
        if error_exit == ErrorExit::Exit {
            exit(1);
        }
    };
    match err {
        SyntaxError => {
            eprintln!("{} contained invalid ruby syntax", source);
            e();
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
            eprintln!("file was: {}", source);
            e();
        }
        IOError(ioe) => {
            eprintln!("IO error occured while running rubyfmt: {:?}, this may indicate a programming error, please file a bug report at https://github.com/penelopezone/rubyfmt/issues/new", ioe);
            e();
        }
        rubyfmt::RichFormatError::OtherRubyError(s) => {
            eprintln!("A ruby error occured: {}, please file a bug report at https://github.com/penelopezone/rubyfmt/issues/new", s);
            exit(1);
        }
    }
}

fn main() {
    let res = rubyfmt::rubyfmt_init();
    if res != rubyfmt::InitStatus::OK as libc::c_int {
        panic!("bad init status");
    }
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && (args[1] == "--help" || args[1] == "-h") {
        eprintln!("{}", include_str!("../README.md"));
        exit(1);
    }

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
            Err(e) => handle_error_from(e, "stdin", ErrorExit::Exit),
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
                    Err(e) => handle_error_from(e, &args[1], ErrorExit::Exit),
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
