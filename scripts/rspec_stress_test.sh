#!/bin/bash
set -ex

run_rspec() {
    (
    cd ./workspace/rubyfmt/rspec-core || exit
    bundle exec rspec
    git reset --hard
    )
}

rm -rf ./workspace
mkdir -p ./workspace/rubyfmt
if [ -z "${GITHUB_REF+x}" ]
then
    echo "not on github"
else
    rm -rf ./workspace/rubyfmt/rspec-core
fi
ls ./workspace/rubyfmt/rspec-core/lib || git clone --depth=1 https://github.com/rspec/rspec-core ./workspace/rubyfmt/rspec-core

(
cd ./workspace/rubyfmt/rspec-core || exit
git reset --hard
bundle
)

FILES=$(find ./workspace/rubyfmt/rspec-core/lib -type f | grep -i '\.rb$')
for FN in $FILES
do
    echo "running rubyfmt on $FN"
    ruby --disable=gems src/rubyfmt.rb "$FN" > ./workspace/rubyfmt/this_one.rb
    ruby --disable=gems src/rubyfmt.rb ./workspace/rubyfmt/this_one.rb > "$FN"

    if [[ "$1" == "--debug" ]]
    then
        run_rspec
    fi
done

run_rspec

# refmt.rb replaces rubyfmt.rb
ruby --disable=gems src/rubyfmt.rb src/rubyfmt.rb > ./workspace/rubyfmt/refmt.rb

FILES=$(find ./workspace/rubyfmt/rspec-core/lib -type f | grep -i '\.rb$')
for FN in $FILES
do
    echo "running rubyfmt on $FN"
    ruby --disable=gems ./workspace/rubyfmt/refmt.rb "$FN" > /./workspace/rubyfmt/this_one.rb
    ruby --disable=gems ./workspace/rubyfmt/refmt.rb /./workspace/rubyfmt/this_one.rb > "$FN"
done

run_rspec
