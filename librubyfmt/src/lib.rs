//#![deny(warnings, missing_copy_implementations)]
use std::ffi::CString;
#[macro_use]
extern crate lazy_static;

use std::io::{BufReader, Write, Cursor};
use std::slice;

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
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

type RubyfmtResult<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

extern "C" {
    pub fn Init_ripper();
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FormatBuffer {
    pub bytes: *const libc::c_char,
    pub count: i64,
}

impl FormatBuffer {
    pub fn into_buf(self) -> &'static [u8] {
        unsafe { slice::from_raw_parts(self.bytes as *const u8, self.count as usize) }
    }

    pub fn into_string(self) -> String {
        unsafe {
            let vec = Vec::from_raw_parts(self.bytes as *mut u8, self.count as usize, self.count as usize);
            String::from_utf8_unchecked(vec)
        }
    }
}

enum InitStatus {
    OK = 0,
    ERROR = 1,
}

#[no_mangle]
pub extern "C" fn rubyfmt_init() -> libc::c_int {
    init_logger();
    unsafe { ruby::ruby_init(); }
    let res = load_ripper();
    if res.is_err() {
        return InitStatus::ERROR as libc::c_int;
    }

    let res = load_rubyfmt();
    if res.is_err() {
        return InitStatus::ERROR as libc::c_int;
    }

    return InitStatus::OK as libc::c_int;
}

pub fn format_buffer(buf: String) -> String {
    let bytes: Vec<libc::c_char> = buf.into_bytes().into_iter().map(|v| v as libc::c_char).collect();
    let len = bytes.len();
    let fb = rubyfmt_format_buffer(FormatBuffer { bytes: bytes.as_ptr(), count: len as i64 });
    fb.into_string()
}

#[no_mangle]
pub extern "C" fn rubyfmt_format_buffer(buf: FormatBuffer) -> FormatBuffer {
    let output_data = Vec::with_capacity(buf.count as usize);
    let mut output = Cursor::new(output_data);
    let tree = run_parser_on(buf);
    if tree.is_err() {
        unsafe {
            ruby::rb_raise(ruby::rb_eRuntimeError, CString::new("oh no").expect("bees").as_ptr());
        }
    }
    let tree = tree.expect("we raised");
    let data = buf.into_buf();
    let res = toplevel_format_program(&mut output, data, tree);
    raise_if_error(res);
    let output_data = output.into_inner().into_boxed_slice();
    let ptr = output_data.as_ptr();
    let len = output_data.len();
    let fb = FormatBuffer { bytes: ptr as *const libc::c_char, count: len as i64 };
    std::mem::forget(output_data);
    return fb;
}


fn load_rubyfmt() -> Result<VALUE, ()> {
    let rubyfmt_program = include_str!("../rubyfmt_lib.rb");
    eval_str(rubyfmt_program)
}

fn load_ripper() -> Result<(), ()> {
    // trick ruby in to thinking ripper is already loaded
    eval_str(r#"
    $LOADED_FEATURES << "ripper.bundle"
    $LOADED_FEATURES << "ripper.so"
    $LOADED_FEATURES << "ripper.rb"
    $LOADED_FEATURES << "ripper/core.rb"
    $LOADED_FEATURES << "ripper/sexp.rb"
    $LOADED_FEATURES << "ripper/filter.rb"
    $LOADED_FEATURES << "ripper/lexer.rb"
    "#)?;

    // init the ripper C module
    unsafe { Init_ripper() };

    //load each ripper program
    eval_str(include_str!("../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper.rb"))?;
    eval_str(include_str!("../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper/core.rb"))?;
    eval_str(include_str!("../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper/lexer.rb"))?;
    eval_str(include_str!("../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper/filter.rb"))?;
    eval_str(include_str!("../ruby_checkout/ruby-2.6.6/ext/ripper/lib/ripper/sexp.rb"))?;

    Ok(())
}

fn eval_str(s: &str) -> Result<VALUE, ()> {
    unsafe {
        let rubyfmt_program_as_c = CString::new(s).expect("it should become a c string");
        let mut state = 0;
        let v = ruby::rb_eval_string_protect(rubyfmt_program_as_c.as_ptr(), &mut state as *mut libc::c_int);
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
            // If the string contains nul, just display the error leading up to
            // the nul bytes
            let c_string = CString::from_vec_unchecked(e.to_string().into_bytes());
            ruby::rb_raise(ruby::rb_eRuntimeError, c_string.as_ptr());
        }
    }
}

fn intern(s: &str) -> ruby::ID {
    unsafe {
        let ruby_string = CString::new(s).expect("it's a string");
        ruby::rb_intern(ruby_string.as_ptr())
    }
}

fn run_parser_on(buf: FormatBuffer) -> Result<VALUE, ()> {
    unsafe {
        let buffer_string = ruby::rb_utf8_str_new(buf.bytes, buf.count);
        let parser_class = eval_str("Parser")?;
        let parser_instance = ruby::rb_funcall(parser_class, intern("new"), 1, buffer_string);
        let tree = ruby::rb_funcall(parser_instance, intern("parse"), 0);
        Ok(tree)
    }
}

fn init_logger() {
    #[cfg(debug_assertions)]
    {
        CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Stderr,
        )
        .unwrap()])
        .unwrap();
        debug!("logger works");
    }
}
