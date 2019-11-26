#![deny(warnings)]
#[macro_use]
extern crate lazy_static;
extern crate regex;

extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::io::{self, BufReader, Write};
use std::str;

pub type RawStatus = i64;

mod comment_block;
mod format;
mod line_metadata;
mod line_tokens;
mod parser_state;
mod ripper_tree_types;
mod ruby_string_pointer;
mod types;

use line_metadata::LineMetadata;
use parser_state::ParserState;
use ruby_string_pointer::RubyStringPointer;

enum Status {
    Ok = 0,
    BadFileName,
    CouldntCreatefile,
    BadJson,
    CouldntWriteFile,
}

#[no_mangle]
pub extern "C" fn format_sexp_tree_to_stdout(
    buf: RubyStringPointer,
    tree: RubyStringPointer,
) -> RawStatus {
    raw_format_program(io::stdout(), buf, tree)
}

#[no_mangle]
pub extern "C" fn format_sexp_tree_to_file(
    filename: RubyStringPointer,
    buf: RubyStringPointer,
    tree: RubyStringPointer,
) -> RawStatus {
    let b = filename.as_buf();
    let filename = match str::from_utf8(b) {
        Ok(x) => x,
        Err(_) => return Status::BadFileName as RawStatus,
    };

    let fp = match File::create(filename) {
        Ok(x) => x,
        Err(_) => return Status::CouldntCreatefile as RawStatus,
    };

    raw_format_program(fp, buf, tree)
}

fn raw_format_program<T: Write>(
    writer: T,
    buf: RubyStringPointer,
    tree: RubyStringPointer,
) -> RawStatus {
    let buf = buf.as_buf();
    let tree = tree.as_buf();

    let res = match toplevel_format_program(writer, buf, tree) {
        Ok(()) => Status::Ok,
        Err(status) => status,
    };

    res as RawStatus
}

fn toplevel_format_program<W: Write>(mut writer: W, buf: &[u8], tree: &[u8]) -> Result<(), Status> {
    let line_metadata = LineMetadata::from_buf(BufReader::new(buf))
        .expect("failed to load line metadata from memory");
    let mut ps = ParserState::new(line_metadata);
    let v: ripper_tree_types::Program = serde_json::from_slice(tree).map_err(|e| {
        println!("{:?}", e);
        Status::BadJson
    })?;

    format::format_program(&mut ps, v);

    ps.write(&mut writer).map_err(|_| Status::CouldntWriteFile)
}
