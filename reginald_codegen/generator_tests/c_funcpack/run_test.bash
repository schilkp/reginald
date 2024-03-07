#!/bin/bash

# Fail on error:
set -e

echo_green() { printf "\033[0;92m" ; echo "$@" ; printf "\033[0m"; }

# Set CWD to location of this script:
cd "${0%/*}"

# Cleanup/Setup
rm -rf output
mkdir -p output

# Run reginald:
echo "Generating..."
cargo run -- gen -i map.yaml -o output/generated.h c-funcpack

# Compile test executeable:
echo "Compiling for host..."
cc test.c -Ioutput -Wall -Wextra -Wpedantic -Wconversion -Warith-conversion -Wint-conversion -Werror -fsanitize=undefined -o output/test

# Run test executeable:
echo "Testing on host..."
./output/test
echo_green "OK!"

# echo "Compile for arm..."
echo "Compiling for arm..."
arm-none-eabi-gcc -c test.c -Ioutput -Wall -Wextra -Wpedantic -Wconversion -Warith-conversion -Wint-conversion -Werror -o output/test
echo_green "OK!"
