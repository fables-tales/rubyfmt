require "mkmf"

$LDFLAGS << " -L./target/release -lrubyfmt "
create_makefile("rubyfmt")
