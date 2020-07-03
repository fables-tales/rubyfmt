#![allow(non_camel_case_types, dead_code)]

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct VALUE(libc::uintptr_t);

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ID(libc::uintptr_t);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ruby_value_type {
    RUBY_T_NONE = 0x00,

    RUBY_T_OBJECT = 0x01,
    RUBY_T_CLASS = 0x02,
    RUBY_T_MODULE = 0x03,
    RUBY_T_FLOAT = 0x04,
    RUBY_T_STRING = 0x05,
    RUBY_T_REGEXP = 0x06,
    RUBY_T_ARRAY = 0x07,
    RUBY_T_HASH = 0x08,
    RUBY_T_STRUCT = 0x09,
    RUBY_T_BIGNUM = 0x0a,
    RUBY_T_FILE = 0x0b,
    RUBY_T_DATA = 0x0c,
    RUBY_T_MATCH = 0x0d,
    RUBY_T_COMPLEX = 0x0e,
    RUBY_T_RATIONAL = 0x0f,

    RUBY_T_NIL = 0x11,
    RUBY_T_TRUE = 0x12,
    RUBY_T_FALSE = 0x13,
    RUBY_T_SYMBOL = 0x14,
    RUBY_T_FIXNUM = 0x15,
    RUBY_T_UNDEF = 0x16,

    RUBY_T_IMEMO = 0x1a,
    RUBY_T_NODE = 0x1b,
    RUBY_T_ICLASS = 0x1c,
    RUBY_T_ZOMBIE = 0x1d,

    RUBY_T_MASK = 0x1f,
}

extern "C" {
    // stuff that we need to compile out rubyfmt
    pub fn ruby_init();
    pub fn ruby_cleanup(_: libc::c_int);
    pub fn rb_eval_string_protect(_: *const libc::c_char, _: *mut libc::c_int) -> VALUE;
    pub fn rb_funcall(_: VALUE, _: ID, _: libc::c_int, ...) -> VALUE;
    pub fn rb_utf8_str_new(_: *const libc::c_char, _: libc::c_long) -> VALUE;
    pub fn rb_str_new_cstr(_: *const libc::c_char) -> VALUE;
    pub fn rb_string_value_cstr(_: VALUE) -> *const libc::c_char;
    pub fn rb_intern(_: *const libc::c_char) -> ID;
    pub fn Init_ripper();
    // Macros/inline functions wrapped as real functions
    pub fn rubyfmt_rstring_ptr(v: VALUE) -> *const libc::c_char;
    pub fn rubyfmt_rstring_len(v: VALUE) -> libc::c_long;
    pub fn rubyfmt_rb_type(v: VALUE) -> ruby_value_type;
    pub fn rubyfmt_rb_num2ll(v: VALUE) -> libc::c_longlong;
    pub fn rubyfmt_rb_ary_len(arr: VALUE) -> libc::c_long;
    pub fn rubyfmt_rb_nil_p(arr: VALUE) -> libc::c_int;

    pub fn rb_protect(f: *const libc::c_void, arg: VALUE, state: *mut libc::c_int) -> VALUE;

    // C statics
    pub static rb_eRuntimeError: VALUE;
    pub static rb_mKernel: VALUE;

    // C functions
    pub fn rb_sym2id(sym: VALUE) -> ID;
    pub fn rb_id2name(id: ID) -> *const libc::c_char;
    pub fn rb_ary_entry(arr: VALUE, idx: libc::c_long) -> VALUE;
    pub fn rb_raise(cls: VALUE, msg: *const libc::c_char);
}
