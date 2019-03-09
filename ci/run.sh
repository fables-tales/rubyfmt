#!/bin/bash
set -ex
./scripts/test.sh
./scripts/rspec_stress_test.sh
