docker-test:
	docker build -t rubyfmt-test . &&  docker run -e GITHUB_TOKEN=$(GITHUB_TOKEN) \
		                                          -e GITHUB_REPOSITORY="samphippen/rubyfmt" \
		                                          -e GITHUB_SHA=$(shell git rev-parse HEAD) \
                                                   -v `pwd`:/github/workspace -it rubyfmt-test
