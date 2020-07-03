//#![deny(warnings, missing_copy_implementations)]
use std::ffi::CString;
#[macro_use]
extern crate lazy_static;

use std::io::{BufReader, Cursor, Write};
use std::slice;
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
mod types;

use file_comments::FileComments;
use parser_state::ParserState;
use ruby::VALUE;

#[cfg(debug_assertions)]
use log::debug;
#[cfg(debug_assertions)]
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};

type RubyfmtResult<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

extern "C" {
    pub fn Init_ripper();
}

pub struct RubyfmtString(Box<str>);

#[derive(Debug, Copy, Clone)]
pub enum InitStatus {
    OK = 0,
    ERROR = 1,
}

pub fn format_buffer(buf: &str) -> String {
    eprintln!("format buffer 1");
    let tree = run_parser_on(buf).expect("the parser works");
    eprintln!("format buffer 2");
    let out_data = vec![];
    eprintln!("format buffer 3");
    let mut output = Cursor::new(out_data);
    eprintln!("format buffer 4");
    let data = buf.as_bytes();
    eprintln!("format buffer 5");
    let res = toplevel_format_program(&mut output, data, tree);
    eprintln!("format buffer 6");
    raise_if_error(res);
    eprintln!("format buffer 7");
    output.flush().expect("flushing works");
    eprintln!("format buffer 8");
    unsafe { String::from_utf8_unchecked(output.into_inner()) }
}

#[no_mangle]
pub extern "C" fn rubyfmt_init() -> libc::c_int {
    init_logger();
    unsafe {
        ruby::ruby_init();
    }
    let res = load_ripper();
    if res.is_err() {
        return InitStatus::ERROR as libc::c_int;
    }

    let res = load_rubyfmt();
    if res.is_err() {
        return InitStatus::ERROR as libc::c_int;
    }

    InitStatus::OK as libc::c_int
}

/// # Safety
/// this function will fail, very badly, if len specifies more bytes than is
/// available in the passed buffer pointer. It will also fail if the passed
/// data isn't utf8.
/// Please don't pass non-utf8 too small buffers.
#[no_mangle]
pub unsafe extern "C" fn rubyfmt_format_buffer(ptr: *const u8, len: usize) -> *mut RubyfmtString {
    let input = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
    let output = format_buffer(input);
    Box::into_raw(Box::new(RubyfmtString(output.into_boxed_str())))
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
        Box::from_raw(rubyfmt_string);
    }
}

fn load_rubyfmt() -> Result<VALUE, ()> {
    let rubyfmt_program = include_str!("../rubyfmt_lib.rb");
    eval_str(rubyfmt_program)
}

fn load_ripper() -> Result<(), ()> {
    // trick ruby in to thinking ripper is already loaded
    eval_str(
        r#"
    $LOADED_FEATURES << "ripper.bundle"
    $LOADED_FEATURES << "ripper.so"
    $LOADED_FEATURES << "ripper.rb"
    $LOADED_FEATURES << "ripper/core.rb"
    $LOADED_FEATURES << "ripper/sexp.rb"
    $LOADED_FEATURES << "ripper/filter.rb"
    $LOADED_FEATURES << "ripper/lexer.rb"
    "#,
    )?;

    // init the ripper C module
    unsafe { Init_ripper() };

    //load each ripper program
    eval_str(include_str!(
        "../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper.rb"
    ))?;
    eval_str(include_str!(
        "../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper/core.rb"
    ))?;
    eval_str(include_str!(
        "../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper/lexer.rb"
    ))?;
    eval_str(include_str!(
        "../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper/filter.rb"
    ))?;
    eval_str(include_str!(
        "../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper/sexp.rb"
    ))?;

    Ok(())
}

fn eval_str(s: &str) -> Result<VALUE, ()> {
    unsafe {
        let rubyfmt_program_as_c = CString::new(s).expect("it should become a c string");
        let mut state = 0;
        let v = ruby::rb_eval_string_protect(
            rubyfmt_program_as_c.as_ptr(),
            &mut state as *mut libc::c_int,
        );
        if state != 0 {
            Err(())
        } else {
            Ok(v)
        }
    }
}

fn toplevel_format_program<W: Write>(writer: &mut W, buf: &[u8], tree: VALUE) -> RubyfmtResult {
    let line_metadata = FileComments::from_buf(BufReader::new(buf))
        .expect("failed to load line metadata from memory");
    let mut ps = ParserState::new(line_metadata);
    let v: ripper_tree_types::Program = de::from_value(tree)?;

    format::format_program(&mut ps, v);

    ps.write(writer)?;
    writer.flush().expect("it flushes");
    Ok(())
}

fn raise_if_error(value: RubyfmtResult) {
    if let Err(e) = value {
        unsafe {
            panic!("got: {:?}", e);
        }
    }
}

fn intern(s: &str) -> ruby::ID {
    unsafe {
        let ruby_string = CString::new(s).expect("it's a string");
        ruby::rb_intern(ruby_string.as_ptr())
    }
}

fn run_parser_on(buf: &str) -> Result<VALUE, ()> {
    unsafe {
        let s = buf;
        eprintln!("parser 1");
        let buffer_string = ruby::rb_utf8_str_new(s.as_ptr() as _, s.len() as i64);
        let mut state = 0;
        let maybe_tree = ruby::rb_protect(my_funcall as _, buffer_string as _, &mut state);
        eprintln!("state: {}", state);
        if state == 0 {
            return Ok(maybe_tree)
        } else {
            let exception = eval_str("puts $!.inspect").expect("this can't fail");
            panic!("failed");
        }
    }
}

unsafe extern "C" fn my_funcall(buffer_string: VALUE) -> VALUE {
    let puts = ruby::rb_funcall(ruby::rb_mKernel, intern("puts"), 1, buffer_string);
    eval_str("STDOUT.flush").expect("flushing stdout works");
    eprintln!("parser 2");
    let parser_class = eval_str("Parser").expect("the parser constant exists");
    eprintln!("parser 3");
    let parser_instance = ruby::rb_funcall(parser_class, intern("new"), 1, buffer_string);
    eprintln!("parser 4");
    let tree = ruby::rb_funcall(parser_instance, intern("parse"), 0);
    eprintln!("parser 5");
    return tree
}


fn init_logger() {
    #[cfg(debug_assertions)]
    {
        TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Stderr)
            .expect("making a term logger");
        debug!("logger works");
    }
}
