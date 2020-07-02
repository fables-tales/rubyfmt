#ifndef RUBYFMT_H
#define RUBYFMT_H


typedef struct _RubyfmtString RubyfmtString;

int rubyfmt_init();
RubyfmtString *rubyfmt_format_buffer(unsigned char* buf, size_t len);
void rubyfmt_string_free(RubyfmtString*);
unsigned char* rubyfmt_string_ptr(const RubyfmtString*);
size_t rubyfmt_string_len(const RubyfmtString*);

int RUBYFMT_INIT_STATUS_OK = 0;
int RUBYFMT_INIT_STATUS_ERROR = 1;

#endif
