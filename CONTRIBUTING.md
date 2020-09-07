# Setup

1. make sure you have a recent-ish rust/cargo toolchain
2. ensure you have at least ruby 2.6 installed

# Day to day tasks

## Running the tests

`./script/test.sh`

in particular `./script/tests/test_fixtures.sh` is what I use for ongoing
development when I'm testing a new feature, which I usually start by committing
a new fixture.

## Doing what CI does:

`./script/ci`

## Autoformatting

Run `make fmt` to cleanup formatting for CI

## Linting

Run `make lint`
