#!/bin/bash

git grep -l "bin\/bash" | xargs shellcheck -x -f gcc
