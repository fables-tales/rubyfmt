# How do I pronounce `rubyfmt`
Ruby format

## How do I use rubyfmt

If you want to see the incredibly janky debug version:

1. have cargo installed
2. type make
3. `ruby --disable=all rubyfmt.rb [the_file_name]`

## Does it work right now?

No

## On RBenv:

RBenv runs what it calls ["shims"](https://github.com/rbenv/rbenv#understanding-shims),
all ruby commands, but most importantly the `ruby` command itself. These
shims impose a penalty to Ruby startup time, because they do a bunch of stuff.
Rubyfmt doesn't need *any* of this stuff to get going. If you'd like the
best performance, make sure you're running ruby by directly going to the binary,
e.g.:

```bash
~/.rbenv/versions/<ruby-version>/bin/ruby --disable=all rubyfmt.rb
```
