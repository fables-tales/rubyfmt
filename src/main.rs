extern crate glob;
extern crate libc;
extern crate rubyfmt;

use std::fs::{metadata, read_to_string, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use glob::glob;

fn rubyfmt_file(file_path: PathBuf) -> io::Result<()> {
    let buffer = read_to_string(file_path.clone())?;
    let res = rubyfmt::format_buffer(buffer);
    let mut file = OpenOptions::new()
        .write(true)
        .open(file_path)
        .expect("file");
    write!(file, "{}", res)?;
    Ok(())
}

fn rubyfmt_dir(path: &String) -> io::Result<()> {
    for entry in glob(&format!("{}/**/*.rb", path)).expect("it exists") {
        let p = entry.expect("should not be null");
        rubyfmt_file(p)?;
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

fn main() {
    let res = rubyfmt::rubyfmt_init();
    if res != rubyfmt::InitStatus::OK as libc::c_int {
        panic!("bad init status");
    }
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("reading frmo stdin to not fail");
        let res = rubyfmt::format_buffer(buffer);
        write!(io::stdout(), "{}", res).expect("write works");
        io::stdout().flush().expect("flush works");
    } else if args.len() == 2 {
        let buffer = read_to_string(args[1].clone()).expect("file exists");
        let res = rubyfmt::format_buffer(buffer);
        write!(io::stdout(), "{}", res).expect("write works");
        io::stdout().flush().expect("flush works");
    } else if args[1] == "-i" {
        let parts = &args[2..args.len()];
        format_parts(parts);
    } else {
        let parts = &args[1..args.len()];
        format_parts(parts);
    }
}
