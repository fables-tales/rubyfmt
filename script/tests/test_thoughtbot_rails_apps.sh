#!/bin/bash
set -euxo pipefail

source "./script/functions.sh"

test_upcase() {
  (
  cd "$(mktemp -d)"
  git clone https://github.com/thoughtbot/upcase --depth=1
  cd upcase
  git checkout 536db4a3bfab0342eb018a85bc97ececff8c77ad
  gem install foreman
  sed -i.bak "s/^ruby.*//" Gemfile
  sed -i.bak "/.*BUNDLED.*/{N;d;}" Gemfile.lock
  bin/setup
  bundle exec rspec
  f_rubyfmt -i app/
  bundle exec rspec
  )
}

test_upcase
