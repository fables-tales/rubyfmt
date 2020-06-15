# How do I pronounce `rubyfmt`
* en: Ruby format
* jp: ルビーフォーマット

## Does it work right now?

No, but it's getting really close

## How do I use it

Build it:
1. Make sure you've got cargo installed
2. Run `make all`
3. Add `export RUBYFMT_USE_RELEASE=1` to your `~/.bashrc` or whichever file gets
   loaded when your shell boots (❤️ zsh users, I just am not one). This'll make
   sure you use the release version of Rubyfmt, and not the debug version, which
   mostly only matters for speed and logging spam.

Rubyfmt supports the following CLI invocations:

* `ruby --disable=all path/to/rubyfmt/rubyfmt.rb filename` to format a file to
  stdout
* `ruby --disable=all path/to/rubyfmt/rubyfmt.rb directory` to format all ruby
  files in that directory in place
* `ruby --disable=all path/to/rubyfmt/rubyfmt.rb -i many file or directory names`
  to format many file and directory names in place

## Command line support
There are two ways to add a terminal command to your environment to make running
rubyfmt easier.

### Shell Alias
If you'd like, you can set up an alias (if you are an RBenv user please read
the section below):

```
alias rubyfmt="ruby --disable=all /absolute/path/to/rubyfmt/rubyfmt.rb"
```

#### On RBenv:

RBenv runs what it calls ["shims"](https://github.com/rbenv/rbenv#understanding-shims),
all ruby commands, but most importantly the `ruby` command itself. These
shims impose a penalty to Ruby startup time, because they do a bunch of stuff.
Rubyfmt doesn't need *any* of this stuff to get going. If you'd like the
best performance, make sure you're running ruby by directly going to the binary,
e.g.:

```bash
~/.rbenv/versions/<ruby-version>/bin/ruby --disable=all rubyfmt.rb
```

### Executable shim script
You can also run an included script to generate a small executable script which you can
place anywhere in your environment's `$PATH`:

```bash
$ /<path-to-rubyfmt-dir>/script/install_shim.sh /usr/local/bin/rubyfmt
```

You can replace `/usr/bin/local` above with any directory in `$PATH`.

## Useful environment variables:

* `RUBYFMT_USE_RELEASE=1`: use release rust compile, much faster + no logging
* `RUBYFMT_DISABLE_SZUSH=1`: disables the backend render queue writer,
  very useful for debugging, literally useless if you're not developing rubyfmt
  itself.

## Editor Support

### Visual Studio Code

Rubyfmt is a supported formatter in the popular
[vscode ruby extension](https://marketplace.visualstudio.com/items?itemName=rebornix.Ruby).
You should follow the above instructions for installing rubyfmt and ensuring an executable is
available in your environment `PATH`. **Note**: You must use the "Executable shim script" method
to get vscode-ruby to work. Once installed, add the following to vscode's `settings.json` file:

``` json
  "ruby.useLanguageServer": true,
  "ruby.format": "rubyfmt",
  "[ruby]": {
      "editor.formatOnSave": true
  },
```