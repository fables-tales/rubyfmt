FROM ubuntu:20.04
RUN apt-get update
RUN apt-get install -y git bison build-essential autoconf ruby curl
RUN apt-get install -y zlib1g-dev clang ruby-dev shellcheck
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN gem install bundler
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y locales
RUN sed -i -e 's/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen && \
    dpkg-reconfigure --frontend=noninteractive locales && \
    update-locale LANG=en_US.UTF-8
ENV LANG en_US.UTF-8
RUN apt-get install -y bison libyaml-dev

ADD ./ /root/rubyfmt.git
