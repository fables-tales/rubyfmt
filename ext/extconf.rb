require "mkmf"

link = "./debug"
if ARGV[0] == "--release"
  link = "./release"
end

$LDFLAGS << " -L#{link} -lrubyfmt "
create_makefile("rubyfmt")
