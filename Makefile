.PHONY: clean

target/rubyfmt.bundle: target/debug/librubyfmt.a
	cp ext/* ./target
	cd target && ruby extconf.rb && make

target/debug/librubyfmt.a: native/src/*.rs native/Cargo.toml
	mkdir -p target/debug
	cd native/ && cargo build && cp target/debug/librubyfmt.a ../target/debug

clean:
	rm -rf target/
