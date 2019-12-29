require "mkmf"

$LDFLAGS << " -L./target/debug -lrubyfmt "
create_makefile("rubyfmt")
