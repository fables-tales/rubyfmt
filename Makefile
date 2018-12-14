docker-test:
	docker build -t rubyfmt-test ./dockerfiles/2.3/ &&  docker run -v `pwd`:/github/workspace -it rubyfmt-test
	docker build -t rubyfmt-test ./dockerfiles/2.5/ &&  docker run -v `pwd`:/github/workspace -it rubyfmt-test
