# Rubyfmt

This is the Rubyfmt release package which contains precompiled binaries for
everything you should need to run Rubyfmt:

1. `rubyfmt`: the main Rubyfmt binary that you can use for autoformatting
2. `rubyfmt-debug`: the Rubyfmt binary compiled with debugging output! Use this
   if you encounter a bug please :)
3. `include/rubyfmt.h`: a C header that you can use to link Rubyfmt in to C programs
4. `lib/librubyfmt.a`: A static lib compiled for linking to C binaries. If you're
    on a mac you'll need to `-framework Foundation -lz` and if you're on linux
    you'll need to `-lcrypt -lm -lpthread -lrt -ldl -lz` to link.
5. `lib/librubyfmt-debug.a`: librubyfmt with debugging logging compiled in.
