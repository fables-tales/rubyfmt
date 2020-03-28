#include "rubyfmt.h"

extern void format_sexp_tree_to_stdout(
    VALUE runtime_error,
    ruby_string_pointer buf,
    VALUE tree
);
extern void format_sexp_tree_to_file(
    VALUE runtime_error,
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

    format_sexp_tree_to_stdout(rb_eRuntimeError, file, tree);
    return Qnil;
}

VALUE rubyfmt_rb_format_to_file(VALUE _mod, VALUE filename, VALUE file_buffer, VALUE tree) {
    ruby_string_pointer fn_p = ruby_string_pointer_from_value(filename);
    ruby_string_pointer buf = ruby_string_pointer_from_value(file_buffer);

    format_sexp_tree_to_file(rb_eRuntimeError, fn_p, buf, tree);
    return Qnil;
}

void Init_rubyfmt() {
    rubyfmt_rb_module_rubyfmt = rb_define_module("Rubyfmt");
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
