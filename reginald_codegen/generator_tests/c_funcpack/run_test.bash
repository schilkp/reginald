#!/bin/bash

# Fail on error:
set -e

echo_green() { printf "\033[0;92m" ; echo "$@" ; printf "\033[0m"; }

# Set CWD to location of this script:
cd "${0%/*}"

# Cleanup/Setup
rm -rf output
mkdir -p output

start_test() {
    echo
    echo_green "$@"
    rm -rf output
    mkdir -p output
}

CFLAGS_COMMON="-Wall -Wextra -Wpedantic -Wconversion -Warith-conversion -Wint-conversion -Werror"

test_generated_code() {
    local EXTRA_CFLAGS="$1"
    local EXTRA_SOURCES="$2"

    # Compile test executeable:
    echo "Compiling for host..."
    cc test.c Unity/unity.c $EXTRA_SOURCES $CFLAGS_COMMON -fsanitize=undefined -o output/test $EXTRA_CFLAGS
    echo_green "OK!"

    # Run test executeable:
    echo "Testing on host..."
    ./output/test
    echo_green "OK!"

    # echo "Compile for arm..."
    echo "Compiling for arm..."
    arm-none-eabi-gcc -c test.c $CFLAGS_COMMON -o output/test $EXTRA_CFLAGS
    echo_green "OK!"
}

# #### C99 #####################################################################

start_test "C99 Test (No generics) - LITTLE ENDIAN"

echo "Generating..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated.h c-funcpack --dont-generate=generic-macros --endian=little
test_generated_code "-DTEST_LITTLE_ENDIAN -DTEST_SKIP_GENERIC_MACROS -std=c99"

start_test "C99 Test (No generics) - BIG ENDIAN"

echo "Generating..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated.h c-funcpack --dont-generate=generic-macros --endian=big
test_generated_code "-DTEST_BIG_ENDIAN -DTEST_SKIP_GENERIC_MACROS -std=c99"

# #### C11 #####################################################################

start_test "C11 Test - LITTLE ENDIAN"

# Run reginald:
echo "Generating..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated.h c-funcpack --endian=little
test_generated_code "-DTEST_LITTLE_ENDIAN -std=c11"

start_test "C11 Test - BIG ENDIAN"

# Run reginald:
echo "Generating..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated.h c-funcpack --endian=big
test_generated_code "-DTEST_BIG_ENDIAN -std=c11"

# #### HEADER + SOURCE #########################################################

start_test "Header and Source Test - LITTLE ENDIAN"

# Run reginald:
echo "Generating header..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated.h c-funcpack \
    --funcs-as-prototypes=true --funcs-static-inline=false

echo "Generating source..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated.c c-funcpack \
    --funcs-static-inline=false          \
    --add-include="generated.h"          \
    --include-guards=false               \
    --doxy-comments=false                \
    --dont-generate=enums                \
    --dont-generate=register-structs     \
    --dont-generate=register-properties  \

test_generated_code "-DTEST_LITTLE_ENDIAN -std=c11" "output/generated.c"

# #### SPLIT HEADERS ###########################################################

start_test "Split Headers Test- LITTLE ENDIAN"

# Run reginald:
echo "Generating enum header..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated_enum.h c-funcpack \
    --only-generate=enums

echo "Generating enum validation header..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated_enum_valid.h c-funcpack \
    --only-generate=enum-validation-funcs \
    --add-include="generated_enum.h"

echo "Generating register struct header..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated_regs.h c-funcpack \
    --only-generate=register-structs \
    --add-include="generated_enum_valid.h"

echo "Generating register props header..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated_reg_props.h c-funcpack \
    --only-generate=register-properties

echo "Generating register conversion function header..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated_regs_conv.h c-funcpack \
    --only-generate=register-conversion-funcs \
    --add-include="generated_reg_props.h"    \
    --add-include="generated_regs.h"    \

echo "Generating generics header..."
cargo run --quiet --color always -- gen -i map.yaml -o output/generated.h c-funcpack \
    --only-generate=generic-macros        \
    --add-include="generated_regs_conv.h" \

test_generated_code "-DTEST_LITTLE_ENDIAN -std=c11"
