#![deny(warnings, missing_copy_implementations)]
extern crate glob;
extern crate libc;
extern crate rubyfmt;

use std::ffi::OsString;
use std::fs::{metadata, read_to_string, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
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

fn rubyfmt_file(file_path: &Path) -> Result<(), FileError> {
    let buffer = read_to_string(&file_path).map_err(FileError::Io)?;
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
            handle_error_from(e, &file_path, ErrorExit::NoExit);
            Ok(())
        }
    }
}

fn rubyfmt_dir(path: &Path) {
    // FIXME: Look for an implementation of glob that actually takes a proper base dir
    for entry in glob(&format!("{}/**/*.rb", path.display())).expect("it exists") {
        let p = entry.expect("should not be null");
        let res = rubyfmt_file(&p);
        if let Err(FileError::SyntaxError) = res {
            eprintln!(
                "warning: {} contains syntax errors, ignoring for now",
                p.display()
            );
        }
    }
}

fn format_parts(parts: &[OsString]) {
    for part in parts {
        if let Ok(md) = metadata(part) {
            if md.is_dir() {
                rubyfmt_dir(part.as_ref());
            } else if md.is_file() {
                rubyfmt_file(part.as_ref()).expect("failed to format file");
            }
        }
    }
}

fn handle_error_from(err: rubyfmt::RichFormatError, source: &Path, error_exit: ErrorExit) {
    use rubyfmt::RichFormatError::*;
    let e = || {
        if error_exit == ErrorExit::Exit {
            exit(1);
        }
    };
    match err {
        SyntaxError => {
            eprintln!("{} contained invalid ruby syntax", source.display());
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
            eprintln!("file was: {}", source.display());
            e();
        }
        IOError(ioe) => {
            eprintln!("IO error occurred while running rubyfmt: {:?}, this may indicate a programming error, please file a bug report at https://github.com/penelopezone/rubyfmt/issues/new", ioe);
            e();
        }
        rubyfmt::RichFormatError::OtherRubyError(s) => {
            eprintln!("A ruby error occurred: {}, please file a bug report at https://github.com/penelopezone/rubyfmt/issues/new", s);
            exit(1);
        }
    }
}

fn main() {
    let res = rubyfmt::rubyfmt_init();
    if res != rubyfmt::InitStatus::OK as libc::c_int {
        panic!(
            "bad init status: {}",
            rubyfmt::ruby::current_exception_as_rust_string()
        );
    }
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let command = args.get(0).and_then(|x| x.to_str());
    match (command, &*args) {
        // Read from stdin
        (_, []) => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .expect("reading from stdin to not fail");
            let res = rubyfmt::format_buffer(&buffer);
            match res {
                Ok(res) => {
                    write!(io::stdout(), "{}", res).expect("write works");
                    io::stdout().flush().expect("flush works");
                }
                Err(e) => handle_error_from(e, Path::new("stdin"), ErrorExit::Exit),
            }
        }
        // In Rust 1.53
        // (Some("--help" | "-h"), _) => {
        (Some("--help"), _) | (Some("-h"), _) => {
            eprintln!("{}", include_str!("../README.md"));
            exit(1);
        }
        // Single file
        (_, [filename]) => {
            if let Ok(md) = metadata(&filename) {
                if md.is_dir() {
                    format_parts(&[filename.clone()])
                } else {
                    let buffer = read_to_string(&filename).expect("file exists");
                    let res = rubyfmt::format_buffer(&buffer);
                    match res {
                        Ok(res) => {
                            write!(io::stdout(), "{}", res).expect("write works");
                            io::stdout().flush().expect("flush works");
                        }
                        Err(e) => handle_error_from(e, filename.as_ref(), ErrorExit::Exit),
                    }
                }
            } else {
                eprintln!("{} does not exist", Path::new(&filename).display());
                exit(1)
            }
        }
        // Multiple files
        (Some("-i"), [_, parts @ ..]) | (_, parts) => {
            format_parts(parts);
        }
    }
}
