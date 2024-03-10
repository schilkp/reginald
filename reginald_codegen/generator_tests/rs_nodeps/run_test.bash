#!/bin/bash

# Fail on error:
set -e

echo_green() { printf "\033[0;92m" ; echo "$@" ; printf "\033[0m"; }

# Set CWD to location of this script:
cd "${0%/*}"

# Cleanup/Setup
rm -rf rs_nodeps_test/src/out.rs

# Run reginald:
echo "Generating..."
cargo run --quiet --color always -p "reginald_codegen" -- gen -i map.yaml -o rs_nodeps_test/src/out.rs rs-nodeps

# Compile + Run test exe:
echo "Testing..."
cd rs_nodeps_test
cargo --color always test
echo_green "OK!"
