.PHONY: clean

debug: target/rubyfmt_debug.bundle

all: release debug

release: target/rubyfmt_release.bundle

target/rubyfmt_debug.bundle: target/debug/librubyfmt.a
	cp ext/* ./target
	cd target && ruby extconf.rb && make

target/rubyfmt_release.bundle: target/release/librubyfmt.a
	cp ext/* ./target
	cd target && ruby extconf.rb --release && make

target/debug/librubyfmt.a: native/src/*.rs native/Cargo.toml
	mkdir -p target/debug
	cd native/ && cargo build && cp target/debug/librubyfmt.a ../target/debug

target/release/librubyfmt.a: native/src/*.rs native/Cargo.toml
	mkdir -p target/release
	cd native/ && cargo build --release && cp target/release/librubyfmt.a ../target/release


clean:
	rm -rf target/
