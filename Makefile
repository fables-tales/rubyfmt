.PHONY: clean clippy lint fmt all release debug

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S), Darwin)
	LDFLAGS=-framework Foundation
endif

ifeq ($(UNAME_S), Linux)
	LDFLAGS=-lcrypt
endif

all: submodules release debug
debug: target/debug/librubyfmt.a
release: target/release/librubyfmt.a

submodules:
	git submodule init
	git submodule update

target/c_main_debug: main.c target/debug/librubyfmt.a
	clang -O3 main.c target/debug/librubyfmt.a $(LDFLAGS) -o $@

target/c_main_release: main.c target/release/librubyfmt.a
	clang -O3 main.c target/release/librubyfmt.a $(LDFLAGS) -o $@

target/release/librubyfmt.a: librubyfmt/src/*.rs librubyfmt/Cargo.toml
	mkdir -p target/release
	cd librubyfmt && cargo build --release
	cp librubyfmt/target/release/librubyfmt.a $@

target/debug/librubyfmt.a: librubyfmt/src/*.rs librubyfmt/Cargo.toml
	mkdir -p target/debug
	cd librubyfmt && cargo build
	cp librubyfmt/target/debug/librubyfmt.a $@

lint: clippy
	./script/lints/lint_fixtures.sh
	./script/lints/lint_scripts.sh
	./script/lints/lint_rust.sh

clean:
	rm -rf target/

fmt:
	cargo fmt
	cd librubyfmt && cargo fmt && git add -u ./

clippy:
	cargo clippy && cd librubyfmt && cargo clippy
