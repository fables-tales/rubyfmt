#!/bin/bash
set -euxo pipefail

source "./script/functions.sh"

test_upcase() {
  (
  cd "$(mktemp -d)"
  git clone https://github.com/thoughtbot/upcase --depth=1
  cd upcase
  gem install foreman
  sed -i.bak "s/^ruby.*//" Gemfile
  sed -i.bak "/.*BUNDLED.*/{N;d;}" Gemfile.lock
  bin/setup
  bundle exec rspec
  export RUBYFMT_USE_RELEASE=1
  f_rubyfmt -i app/
  bundle exec rspec
  )
}

test_upcase
