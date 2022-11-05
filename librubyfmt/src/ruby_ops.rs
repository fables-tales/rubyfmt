use crate::file_comments::FileComments;
use crate::ruby::*;

pub fn setup_ruby() -> Result<(), ()> {
    unsafe {
        let res = ruby_setup();
        if res == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

// Safety: This function expects an initialized Ruby VM
pub unsafe fn load_rubyfmt() -> Result<(), ()> {
    let rubyfmt_program = include_str!("../rubyfmt_lib.rb");
    eval_str(rubyfmt_program)?;
    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub struct Parser(VALUE);

#[derive(Debug, Clone)]
pub enum ParseError {
    SyntaxError,
    OtherRubyError(String),
}

impl Parser {
    unsafe extern "C" fn real_run_parser(parser_instance: VALUE) -> VALUE {
        rb_funcall(parser_instance, intern!("parse"), 0)
    }

    pub fn new(buf: &str) -> Self {
        unsafe {
            let buffer_string = rb_utf8_str_new(buf.as_ptr() as _, buf.len() as libc::c_long);
            let parser_class = rb_const_get_at(rb_cObject, intern!("Parser"));
            let parser_instance = rb_funcall(parser_class, intern!("new"), 1, buffer_string);
            Parser(parser_instance)
        }
    }

    pub fn parse(self) -> Result<(RipperTree, FileComments, Option<&'static str>), ParseError> {
        let mut state = 0;
        let maybe_ret_tuple =
            unsafe { rb_protect(Parser::real_run_parser as _, self.0 as _, &mut state) };
        if state == 0 {
            if maybe_ret_tuple != Qnil {
                let ret_tuple = unsafe { ruby_array_to_slice(maybe_ret_tuple) };
                if let [tree, comments, lines, last_lineno, end_contents] = ret_tuple {
                    let fc = FileComments::from_ruby_hash(*comments, *lines, *last_lineno);
                    let end_contents = unsafe {
                        if rubyfmt_rb_nil_p(*end_contents) != 0 {
                            None
                        } else {
                            Some(ruby_string_to_str(*end_contents))
                        }
                    };
                    Ok((RipperTree::new(*tree), fc, end_contents))
                } else {
                    panic!(
                        "expected return tuple to match expected, actually got: {}",
                        ret_tuple.len(),
                    )
                }
            } else {
                Err(ParseError::SyntaxError)
            }
        } else {
            let s = current_exception_as_rust_string();
            Err(ParseError::OtherRubyError(s))
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RipperTree(VALUE);

impl RipperTree {
    pub fn new(v: VALUE) -> Self {
        RipperTree(v)
    }

    pub fn into_value(self) -> VALUE {
        self.0
    }
}
