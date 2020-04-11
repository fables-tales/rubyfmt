#include "rubyfmt.h"

extern void init_logger();
extern void format_sexp_tree_to_stdout(ruby_string_pointer buf, VALUE tree);
extern void format_sexp_tree_to_file(
    ruby_string_pointer filename,
    ruby_string_pointer buf,
    VALUE tree
);

VALUE rubyfmt_rb_module_rubyfmt = Qnil;

ruby_string_pointer ruby_string_pointer_from_value(VALUE string) {
    ruby_string_pointer ret =  {StringValuePtr(string), RSTRING_LEN(string)};
    return ret;
}

VALUE rubyfmt_rb_format_to_stdout(VALUE _mod, VALUE file_buffer, VALUE tree) {
    ruby_string_pointer file = ruby_string_pointer_from_value(file_buffer);

    format_sexp_tree_to_stdout(file, tree);
    return Qnil;
}

VALUE rubyfmt_rb_format_to_file(VALUE _mod, VALUE filename, VALUE file_buffer, VALUE tree) {
    ruby_string_pointer fn_p = ruby_string_pointer_from_value(filename);
    ruby_string_pointer buf = ruby_string_pointer_from_value(file_buffer);

    format_sexp_tree_to_file(fn_p, buf, tree);
    return Qnil;
}

char *rubyfmt_rstring_ptr(VALUE s) {
  return RSTRING_PTR(s);
}

long rubyfmt_rstring_len(VALUE s) {
  return RSTRING_LEN(s);
}

enum ruby_value_type rubyfmt_rb_type(VALUE v) {
  return rb_type(v);
}

long long rubyfmt_rb_num2ll(VALUE v) {
  return RB_NUM2LL(v);
}

long rubyfmt_rb_ary_len(VALUE v) {
  return rb_array_len(v);
}

int rubyfmt_rb_nil_p(VALUE v) {
  return RB_NIL_P(v);
}

void Init_rubyfmt() {
    rubyfmt_rb_module_rubyfmt = rb_define_module("Rubyfmt");
    init_logger();
    rb_define_module_function(
        rubyfmt_rb_module_rubyfmt,
        "format_to_stdout",
        rubyfmt_rb_format_to_stdout,
        2
    );
    rb_define_module_function(
        rubyfmt_rb_module_rubyfmt,
        "format_to_file",
        rubyfmt_rb_format_to_file,
        3
    );
}

void Init_rubyfmt_debug() {
    Init_rubyfmt();
}

void Init_rubyfmt_release() {
    Init_rubyfmt();
}
