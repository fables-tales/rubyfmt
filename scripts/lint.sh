#!/bin/bash

git grep -l "bin\/bash" | xargs shellcheck -f gcc
