#ifndef __RUBYFMT_H
#define __RUBYFMT_H

#include <ruby.h>
#include <stdint.h>

typedef struct _ruby_string_pointer {
    const char* bytes;
    int64_t length;
} ruby_string_pointer;

#endif
