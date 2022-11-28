#![deny(warnings, missing_copy_implementations)]
#![allow(clippy::upper_case_acronyms, clippy::enum_variant_names)]

use serde::de::value;
use std::io::{Cursor, Write};
use std::slice;
use std::str;

#[macro_use]
extern crate lazy_static;

#[cfg(all(feature = "use_jemalloc", not(target_env = "msvc")))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub type RawStatus = i64;

mod comment_block;
mod delimiters;
mod file_comments;
mod format;
mod heredoc_string;
mod intermediary;
mod line_metadata;
mod line_tokens;
mod parser_state;
mod render_queue_writer;
mod render_targets;
mod ripper_tree_types;
mod types;

use file_comments::FileComments;
use parser_state::BaseParserState;
use ruby_ops::{ParseError, Parser, RipperTree, RubyComments};

pub struct RubyfmtString(Box<str>);

#[derive(Debug)]
pub enum RichFormatError {
    SyntaxError,
    RipperParseFailure(value::Error),
    IOError(std::io::Error),
    OtherRubyError(String),
}

impl RichFormatError {
    pub fn as_exit_code(&self) -> i32 {
        self.as_format_error() as i32
    }

    fn as_format_error(&self) -> FormatError {
        match self {
            RichFormatError::SyntaxError => FormatError::SyntaxError,
            RichFormatError::RipperParseFailure(_) => FormatError::RipperParseFailure,
            RichFormatError::IOError(_) => FormatError::IOError,
            RichFormatError::OtherRubyError(_) => FormatError::OtherRubyError,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FormatError {
    OK = 0,
    SyntaxError = 1,
    RipperParseFailure = 2,
    IOError = 3,
    OtherRubyError = 4,
    // Diffs are only necessary in --check mode
    DiffDetected = 5,
}

pub fn format_buffer(buf: &str) -> Result<String, RichFormatError> {
    let (tree, file_comments, end_data) = run_parser_on(buf)?;
    let out_data = vec![];
    let mut output = Cursor::new(out_data);
    toplevel_format_program(
        &mut output,
        tree,
        FileComments::new(file_comments),
        end_data,
    )?;
    output.flush().expect("flushing to a vec should never fail");
    Ok(String::from_utf8(output.into_inner()).expect("we never write invalid UTF-8"))
}

/// # Safety
/// this function will fail, very badly, if len specifies more bytes than is
/// available in the passed buffer pointer. It will also fail if the passed
/// data isn't utf8.
/// Please don't pass non-utf8 too small buffers.
#[no_mangle]
pub unsafe extern "C" fn rubyfmt_format_buffer(
    ptr: *const u8,
    len: usize,
    err: *mut i64,
) -> *mut RubyfmtString {
    let input = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
    let output = format_buffer(input);
    match output {
        Ok(o) => {
            *err = FormatError::OK as i64;
            Box::into_raw(Box::new(RubyfmtString(o.into_boxed_str())))
        }
        Err(e) => {
            *err = e.as_format_error() as i64;
            std::ptr::null::<RubyfmtString>() as _
        }
    }
}

#[no_mangle]
pub extern "C" fn rubyfmt_string_ptr(s: &RubyfmtString) -> *const u8 {
    s.0.as_ptr()
}

#[no_mangle]
pub extern "C" fn rubyfmt_string_len(s: &RubyfmtString) -> usize {
    s.0.len()
}

#[no_mangle]
extern "C" fn rubyfmt_string_free(rubyfmt_string: *mut RubyfmtString) {
    unsafe {
        drop(Box::from_raw(rubyfmt_string));
    }
}

pub fn toplevel_format_program<W: Write>(
    writer: &mut W,
    tree: RipperTree,
    file_comments: FileComments,
    end_data: Option<&str>,
) -> Result<(), RichFormatError> {
    let mut ps = BaseParserState::new(file_comments);
    let v: ripper_tree_types::Program =
        ruby_ops::de::from_value(tree).map_err(RichFormatError::RipperParseFailure)?;

    format::format_program(&mut ps, v, end_data);

    ps.write(writer).map_err(RichFormatError::IOError)?;
    writer.flush().map_err(RichFormatError::IOError)?;
    Ok(())
}

fn run_parser_on(buf: &str) -> Result<(RipperTree, RubyComments, Option<&str>), RichFormatError> {
    Parser::new(buf).parse().map_err(|e| match e {
        ParseError::SyntaxError => RichFormatError::SyntaxError,
        ParseError::OtherRubyError(s) => RichFormatError::OtherRubyError(s),
    })
}
