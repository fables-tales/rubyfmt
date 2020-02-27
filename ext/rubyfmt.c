#include "rubyfmt.h"

extern int64_t format_sexp_tree_to_stdout(ruby_string_pointer buf, ruby_string_pointer tree);
extern int64_t format_sexp_tree_to_file(
     ruby_string_pointer filename,
     ruby_string_pointer buf,
     ruby_string_pointer tree
);

VALUE rubyfmt_rb_module_rubyfmt = Qnil;

ruby_string_pointer ruby_string_pointer_from_value(VALUE string) {
    ruby_string_pointer ret =  {StringValuePtr(string), RSTRING_LEN(string)};
    return ret;
}

VALUE rubyfmt_rb_format_to_stdout(VALUE _mod, VALUE file_buffer, VALUE tree_json) {
    ruby_string_pointer file = ruby_string_pointer_from_value(file_buffer);
    ruby_string_pointer tree = ruby_string_pointer_from_value(tree_json);

    int64_t status = format_sexp_tree_to_stdout(file, tree);

    if (status != 0) {
        rb_raise(rb_eRuntimeError, "Error code %lli", status);
    }

    return Qnil;
}

VALUE rubyfmt_rb_format_to_file(VALUE _mod, VALUE filename, VALUE file_buffer, VALUE tree_json) {
    ruby_string_pointer fn_p = ruby_string_pointer_from_value(filename);
    ruby_string_pointer buf = ruby_string_pointer_from_value(file_buffer);
    ruby_string_pointer tree = ruby_string_pointer_from_value(tree_json);

    int64_t status = format_sexp_tree_to_file(fn_p, buf, tree);

    if (status != 0) {
        rb_raise(rb_eRuntimeError, "Error code %lli", status);
    }

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
