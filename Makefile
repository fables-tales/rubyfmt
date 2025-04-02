.PHONY: clean clippy lint fmt all release debug ubuntu_shell

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S), Darwin)
	LDFLAGS=-framework Foundation -framework Security
endif

ifeq ($(UNAME_S), Linux)
	LDFLAGS=-lcrypt -lm -lpthread -lrt -ldl
endif

LDFLAGS +=  -lz

all: release debug

debug:
	bash -c "find target/debug | grep -i 'librubyfmt-.*\.a' | xargs rm; exit 0"
	cargo build

release:
	bash -c "find target/release | grep -i 'librubyfmt-.*\.a' | xargs rm; exit 0"
	cargo build --release

ubuntu_shell:
	docker build -t rubyfmt_testing_container:$(shell git rev-parse HEAD) -f ./dockerfiles/build.Dockerfile ./
	docker run -it rubyfmt_testing_container:$(shell git rev-parse HEAD) bash

lint: clippy
	./script/lints/lint_fixtures.sh
	./script/lints/lint_scripts.sh

clean:
	rm -rf target/

fmt:
	cargo fmt

clippy:
	cargo clippy
