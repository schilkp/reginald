#!/bin/bash

# Fail on error:
set -e

echo_green() { printf "\033[0;92m" ; echo "$@" ; printf "\033[0m"; }

# Set CWD to location of this script:
cd "${0%/*}"

# Cleanup/Setup
rm -rf output
mkdir -p output

# #### C99 #####################################################################

echo
echo_green "C99 Test (No generics)"
echo

# Run reginald:
echo "Generating..."
rm -rf output
mkdir -p output
cargo run --quiet --color always -- gen -i map.yaml -o output/generated.h c-funcpack --generate-generic-macros=false

# Compile test executeable:
echo "Compiling for host..."
cc test.c -std=c99 -Ioutput -Wall -Wextra -Wpedantic -Wconversion -Warith-conversion -Wint-conversion -Werror -fsanitize=undefined -o output/test
echo_green "OK!"

# Run test executeable:
echo "Testing on host..."
./output/test
echo_green "OK!"

# echo "Compile for arm..."
echo "Compiling for arm..."
arm-none-eabi-gcc -c -std=c99 test.c -Ioutput -Wall -Wextra -Wpedantic -Wconversion -Warith-conversion -Wint-conversion -Werror -o output/test
echo_green "OK!"

# #### C11 #####################################################################

echo
echo_green "C11 Test"
echo

# Run reginald:
echo "Generating..."
rm -rf output
mkdir -p output
cargo run --quiet --color always -- gen -i map.yaml -o output/generated.h c-funcpack

# Compile test executeable:
echo "Compiling for host..."
cc test.c -std=c11 -Ioutput -Wall -Wextra -Wpedantic -Wconversion -Warith-conversion -Wint-conversion -Werror -fsanitize=undefined -DGENERIC_REGINALD_FUNCS -o output/test
echo_green "OK!"

# Run test executeable:
echo "Testing on host..."
./output/test
echo_green "OK!"

# echo "Compile for arm..."
echo "Compiling for arm..."
arm-none-eabi-gcc -c -std=c11 test.c -Ioutput -Wall -Wextra -Wpedantic -Wconversion -Warith-conversion -Wint-conversion -Werror -DGENERIC_REGINALD_FUNCS -o output/test
echo_green "OK!"
