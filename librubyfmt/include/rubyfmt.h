#ifndef RUBYFMT_H
#define RUBYFMT_H

typedef struct _FormatBuffer {
    const char* bytes;
    int64_t count;
} FormatBuffer;

int rubyfmt_init();
FormatBuffer rubyfmt_format_buffer(FormatBuffer buf);

#endif
