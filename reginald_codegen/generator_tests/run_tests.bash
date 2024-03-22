#!/bin/bash

# Fail on error:
set -e

# Utils:
echo_red() { printf "\033[0;31m" ; echo "$@" ; printf "\033[0m"; }
echo_cyan() { printf "\033[0;36m" ; echo "$@" ; printf "\033[0m"; }
echo_green() { printf "\033[0;92m" ; echo "$@" ; printf "\033[0m"; }
fail_count=0
fail() {
    echo_red "TEST FAIL"
    fail_count=$((fail_count+1))
}

# Set CWD to location of this script:
cd "${0%/*}"

# Run tests:
echo_cyan "=== COMPILE ==="
cargo build
echo_green "DONE"

echo
echo_cyan "=== TESTING C FUNCPACK ==="
bash ./c_funcpack/run_test.bash || fail

echo
echo_cyan "=== TESTING RS STRUCTS ==="
bash ./rs_structs/run_test.bash || fail

# Report:
if [ "$fail_count" -eq "0" ]; then
    echo_green "ALL OK:)";
else
    echo_red "$fail_count TEST FAILURE(S)!";
fi
exit $fail_count
