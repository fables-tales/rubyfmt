#!/bin/bash
set -ex
source "./script/functions.sh"


make

test_rspec_repo() {
    (
    ls "tmp/$1/lib" || git clone --depth=1 "https://github.com/rspec/$1" "tmp/$1"
    cd "tmp/$1"
    git reset --hard
    bundle
    cd ../..

    FILES=$(find "tmp/$1/lib" -type f | grep -i '\.rb$')
    for FN in $FILES
    do
        echo "running rubyfmt on $FN"
        TMPFILE=$(mktemp) || exit 1
        f_rubyfmt "$FN" > "$TMPFILE"
        f_rubyfmt "$TMPFILE" > "$FN"
    done
    cd "tmp/$1"
    bundle exec rspec --exclude-pattern ./spec/integration/persistence_failures_spec.rb
    git reset --hard
    f_rubyfmt -i "lib/"
    git status
    bundle exec rspec --exclude-pattern ./spec/integration/persistence_failures_spec.rb
    git reset --hard
    cd ../../
    )
}

test_rspec_repo_incrementally() {
    (
    ls tmp/"$1"/lib || git clone --depth=1 "https://github.com/rspec/$1" "tmp/$1"
    cd "tmp/$1"
    git reset --hard
    bundle
    cd ../..

    FILES=$(find "tmp/$1/lib" -type f | grep -i '\.rb$')
    for FN in $FILES
    do
        echo "running rubyfmt on $FN"
        f_rubyfmt "$FN" > /tmp/this_one.rb
        f_rubyfmt /tmp/this_one.rb > "$FN"
        cd "tmp/$1"
        bundle exec rspec --exclude-pattern ./spec/integration/persistence_failures_spec.rb
        git reset --hard
        cd ../../
    done
    )
}

test_rspec_repo rspec-core &
test_rspec_repo rspec-mocks &
test_rspec_repo rspec-expectations &
wait
