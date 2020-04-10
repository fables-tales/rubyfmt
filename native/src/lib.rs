#![deny(warnings, missing_copy_implementations)]
#[macro_use]
extern crate lazy_static;


use std::fs::File;
use std::io::{self, BufReader, Write};
use std::str;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub type RawStatus = i64;

mod breakable_entry;
mod comment_block;
mod de;
mod delimiters;
mod file_comments;
mod format;
mod intermediary;
mod line_metadata;
mod line_tokens;
mod parser_state;
mod render_queue_writer;
mod ripper_tree_types;
mod ruby;
mod ruby_string_pointer;
mod types;

use file_comments::FileComments;
use parser_state::ParserState;
use ruby::VALUE;
use ruby_string_pointer::RubyStringPointer;

#[cfg(debug_assertions)]
use simplelog::{CombinedLogger, TermLogger, LevelFilter, Config, TerminalMode};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[no_mangle]
pub extern "C" fn format_sexp_tree_to_stdout(buf: RubyStringPointer, tree: VALUE) {
    raise_if_error(raw_format_program(None, buf, tree));
}

#[no_mangle]
pub extern "C" fn format_sexp_tree_to_file(
    filename: RubyStringPointer,
    buf: RubyStringPointer,
    tree: VALUE,
) {
    raise_if_error(raw_format_program(Some(filename), buf, tree))
}

fn raw_format_program(
    filename: Option<RubyStringPointer>,
    buf: RubyStringPointer,
    tree: VALUE,
) -> Result {
    let buf = buf.into_buf();
    let mut file;
    let mut stdout = io::stdout();
    let writer: &mut dyn Write = match filename {
        Some(fp) => {
            // FIXME: We should try to do an OsStr here
            let name = str::from_utf8(fp.into_buf())?;
            file = File::create(name)?;
            &mut file
        }
        None => &mut stdout,
    };

    toplevel_format_program(writer, buf, tree)
}

fn toplevel_format_program<W: Write>(mut writer: W, buf: &[u8], tree: VALUE) -> Result {
    #[cfg(debug_assertions)]
    CombinedLogger::init(
        vec![
        TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Stderr).unwrap(),
        ]
    ).unwrap();

    let line_metadata = FileComments::from_buf(BufReader::new(buf))
        .expect("failed to load line metadata from memory");
    let mut ps = ParserState::new(line_metadata);
    let v: ripper_tree_types::Program = de::from_value(tree)?;

    format::format_program(&mut ps, v);

    ps.write(&mut writer)?;
    Ok(())
}

fn raise_if_error(value: Result) {
    use std::ffi::CString;

    if let Err(e) = value {
        unsafe {
            // If the string contains nul, just display the error leading up to
            // the nul bytes
            let c_string = CString::from_vec_unchecked(e.to_string().into_bytes());
            ruby::rb_raise(ruby::rb_eRuntimeError, c_string.as_ptr());
        }
    }
}
