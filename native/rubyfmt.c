#include <ruby.h>
#include <stdint.h>
#include <stdbool.h>

extern void* writer_open_handle_or_panic(char* name_bytes, int64_t name_length);
extern void* writer_open_stdout();
extern void* writer_file_writer_write_bytes_or_panic(
    void* writer,
    char* bytes,
    int64_t length
);

extern void* writer_stdout_writer_write_bytes_or_panic(
    void* writer,
    char* bytes,
    int64_t length
);

VALUE rubyfmt_rb_module_rubyfmt = Qnil;

void* current_writer;
bool use_stdout;

VALUE handle_next_token_from_intermediary(VALUE token, VALUE data, int argc, VALUE* argv) {
    VALUE string = rb_funcall(token, rb_intern("to_s"), 0);
    char* string_data = StringValuePtr(string);
    int64_t string_length = RSTRING_LEN(string);
    if (use_stdout) {
        writer_stdout_writer_write_bytes_or_panic(current_writer, string_data, string_length);
    } else {
        writer_file_writer_write_bytes_or_panic(current_writer, string_data, string_length);
    }
    return Qnil;
}

// we use filename == nil to represent stdout because I'm a horrible programmer
VALUE rubyfmt_rb_write_intermediary(VALUE klass, VALUE filename, VALUE intermediary) {
    if (filename == Qnil) {
        use_stdout = true;
        current_writer = writer_open_stdout();
    } else {
        use_stdout = false;
        char* name_bytes = StringValuePtr(filename);
        int64_t name_length = RSTRING_LEN(filename);
        current_writer = writer_open_handle_or_panic(name_bytes, name_length);
    }
    rb_block_call(intermediary, rb_intern("each"), 0, NULL, handle_next_token_from_intermediary, Qnil);
    return Qnil;
}

void Init_rubyfmt() {
    rubyfmt_rb_module_rubyfmt = rb_define_module("Rubyfmt");
    rb_define_module_function(
        rubyfmt_rb_module_rubyfmt,
        "write_intermediary",
        rubyfmt_rb_write_intermediary,
        2
    );
}
