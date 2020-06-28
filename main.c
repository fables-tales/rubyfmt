#include <stdio.h>
#include <stdlib.h>
#include "./librubyfmt/include/rubyfmt.h"

int main() {
    int buf_size = 1024;
    int bytes_read = 0;
    char* buf = malloc(sizeof(char)*buf_size);
    int nread = fread(buf, sizeof(char), buf_size, stdin);
    while (nread == buf_size) {
        bytes_read += nread;
        int old_buf_size = buf_size;
        buf_size *= 2;
        buf = realloc(buf, sizeof(char)*buf_size);
        nread = fread(buf+bytes_read, sizeof(char), buf_size-old_buf_size, stdin);
    }
    bytes_read += nread;

    FormatBuffer fb = { .bytes = buf, .count = bytes_read };

    int res = rubyfmt_init();
    if (res != RUBYFMT_INIT_STATUS_OK) {
        fprintf(stderr, "failed to init\n");
        exit(1);
    }
    FormatBuffer out = rubyfmt_format_buffer(fb);
    fwrite(out.bytes, sizeof(char), out.count, stdout);
}
