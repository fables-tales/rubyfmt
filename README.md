# Rubyfmt - 0.1, the one where it doesn't break RSpec

Rubyfmt is a Ruby autoformatter in the style of
[gofmt](https://golang.org/cmd/gofmt/).  Unlike rubocop, it is intended to
*only* be a formatter, and not any kind of deeper analysis tool.

Rubyfmt is currently "functional" in the sense that it can execute over the
entirety of [rspec/rspec-core](https://github.com/rspec/rspec-core) and not
break any of the tests.

**Rubyfmt currently just outputs valid ruby, and I haven't done much in the way
of making it style things in any way that remotely mirrors what makes sense for
ruby. As such isn't yet really fit for day to day usage. If you'd like to try it out
though, you can check out the section below**

## Installation

Rubyfmt is a standalone script that only loads the standard library of Ruby,
as such it is not packaged as a gem. It is intended to be in your editor's save
hook and run really fast.

I suggest:

* Download `src/rubyfmt.rb` to `~/bin`
* Add `~/bin` to your PATH (e.g. `echo "$HOME/bin:$PATH" >> ~/.bash_profile`)
* Set your editor to run `rubyfmt file_name > file_name` on save.


## Contributing

Rubyfmt considers any file going through the formatter, and coming out the other
side with changed semantics to be a bug. Please
[file an issue](https://github.com/samphippen/rubyfmt/issues/new) or [open a pull request](https://github.com/samphippen/rubyfmt/compare)

At this stage things are too early for us to accept PRs that affect output
styling.

I will happily accept plugins that make Rubyfmt work with your favourite editor,
or improve the CLI usability.

## What's coming?

The next thing I'm gonna do is make Rubyfmt self hosting, which will naturally involve me updating how it formats things in a way that I don't hate.
