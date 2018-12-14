FROM ruby:2.3

ADD Gemfile /app/Gemfile
ADD Gemfile.lock /app/Gemfile.lock

WORKDIR /app

RUN bundle install

CMD cd /github/workspace && ./ci/run.sh
