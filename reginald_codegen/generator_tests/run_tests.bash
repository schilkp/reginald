#!/bin/bash

# Fail on error:
set -e

echo_red() { printf "\033[0;31m" ; echo "$@" ; printf "\033[0m"; }
echo_cyan() { printf "\033[0;36m" ; echo "$@" ; printf "\033[0m"; }
echo_green() { printf "\033[0;92m" ; echo "$@" ; printf "\033[0m"; }

# Set CWD to location of this script:
cd "${0%/*}"

echo_cyan "=== COMPILE ==="
cargo build
echo_green "DONE"

# Run tests:
echo
echo_cyan "=== TESTING C FUNCPACK ==="
bash ./c_funcpack/run_test.bash || echo_red "TEST FAIL"
echo_green "DONE"

echo
echo_cyan "=== TESTING RS STRUCT NO DEPS==="
bash ./rs_struct_no_deps/run_test.bash || echo_red "TEST FAIL"
echo_green "DONE"
