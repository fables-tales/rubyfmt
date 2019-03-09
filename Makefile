docker-test:
	docker build -t rubyfmt-test ./dockerfiles/2.3/ &&  docker run -e LANG="en_US.UTF-8" -e LANGUAGE="en_US:en" -e LC_ALL="en_US.UTF-8" -v `pwd`:/github/workspace -it rubyfmt-test
	docker build -t rubyfmt-test ./dockerfiles/2.5/ &&  docker run -e LANG="en_US.UTF-8" -e LANGUAGE="en_US:en" -e LC_ALL="en_US.UTF-8" -v `pwd`:/github/workspace -it rubyfmt-test
