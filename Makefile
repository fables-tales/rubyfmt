.PHONY: docker-test
all: build/rubyfmt.rb docker-test

build/rubyfmt.rb: src/requires.rb src/line_metadata.rb src/line_parts.rb src/line.rb src/parser_state.rb src/parser.rb src/format.rb src/main.rb
	cat $? > $@

docker-test:
	docker build -t rubyfmt-test ./dockerfiles/2.3/ &&  docker run -e LANG="en_US.UTF-8" -e LANGUAGE="en_US:en" -e LC_ALL="en_US.UTF-8" -v `pwd`:/github/workspace -it rubyfmt-test
	docker build -t rubyfmt-test ./dockerfiles/2.5/ &&  docker run -e LANG="en_US.UTF-8" -e LANGUAGE="en_US:en" -e LC_ALL="en_US.UTF-8" -v `pwd`:/github/workspace -it rubyfmt-test
	docker build -t rubyfmt-test ./dockerfiles/2.6/ &&  docker run -e LANG="en_US.UTF-8" -e LANGUAGE="en_US:en" -e LC_ALL="en_US.UTF-8" -v `pwd`:/github/workspace -it rubyfmt-test
