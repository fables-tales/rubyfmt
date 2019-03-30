FROM ruby:2.3

RUN apt-get update && \
        DEBIAN_FRONTEND=noninteractive apt-get install -y \
        bc \
        locales \
        shellcheck

RUN sed -i -e 's/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/' /etc/locale.gen && \
    locale-gen
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8
CMD cd /github/workspace && ./ci/run.sh
