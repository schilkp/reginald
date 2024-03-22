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
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated.h c-funcpack \
    --dont-generate=generic-macros \
    --endian=little \
    --endianess-in-names=false
test_generated_code "-DTEST_LITTLE_ENDIAN -DTEST_SKIP_GENERIC_MACROS -std=c99 -fanalyzer"

start_test "C99 Test (No generics) - BIG ENDIAN"

echo "Generating..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated.h c-funcpack \
    --dont-generate=generic-macros \
    --endian=big \
    --endianess-in-names=false
test_generated_code "-DTEST_BIG_ENDIAN -DTEST_SKIP_GENERIC_MACROS -std=c99 -fanalyzer"

# #### C11 #####################################################################

start_test "C11 Test - LITTLE ENDIAN"

# Run reginald:
echo "Generating..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated.h c-funcpack \
    --endian=little \
    --endianess-in-names=false
test_generated_code "-DTEST_LITTLE_ENDIAN -std=c11"

start_test "C11 Test - BIG ENDIAN"

# Run reginald:
echo "Generating..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated.h c-funcpack \
    --endian=big \
    --endianess-in-names=false
test_generated_code "-DTEST_BIG_ENDIAN -std=c11"

# #### HEADER + SOURCE #########################################################

start_test "Header and Source Test - LITTLE ENDIAN"

# Run reginald:
echo "Generating header..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated.h c-funcpack \
    --endian=little \
    --endianess-in-names=false \
    --funcs-as-prototypes=true --funcs-static-inline=false

echo "Generating source..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated.c c-funcpack \
    --endian=little                         \
    --endianess-in-names=false               \
    --funcs-static-inline=false             \
    --add-include="generated.h"             \
    --include-guards=false                  \
    --doxy-comments=false                   \
    --only-generate=enum-validation-funcs   \
    --only-generate=struct-conversion-funcs \

test_generated_code "-DTEST_LITTLE_ENDIAN -std=c11" "output/generated.c"

# #### SPLIT HEADERS ###########################################################

start_test "Split Headers Test- LITTLE ENDIAN"

# Run reginald:
echo "Generating enum header..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated_enum.h c-funcpack \
    --endian=little           \
    --endianess-in-names=false \
    --only-generate=enums

echo "Generating enum validation header..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated_enum_valid.h c-funcpack \
    --endian=little                       \
    --endianess-in-names=false             \
    --only-generate=enum-validation-funcs \
    --add-include="generated_enum.h"

echo "Generating struct header..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated_structs.h c-funcpack \
    --endian=little            \
    --endianess-in-names=false  \
    --only-generate=structs    \
    --add-include="generated_enum_valid.h"

echo "Generating struct conversion func header..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated_struct_conv.h c-funcpack \
    --endian=little                         \
    --endianess-in-names=false               \
    --only-generate=struct-conversion-funcs \
    --add-include="generated_structs.h"

echo "Generating register properties header..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated_reg_props.h c-funcpack \
    --endian=little                         \
    --endianess-in-names=false               \
    --only-generate=register-properties     \
    --add-include="generated_struct_conv.h"

echo "Generating generics header..."
cargo run --quiet --color always -- gen -i ../map.yaml -o output/generated.h c-funcpack \
    --endian=little                       \
    --endianess-in-names=false             \
    --only-generate=generic-macros        \
    --add-include="generated_reg_props.h" \

test_generated_code "-DTEST_LITTLE_ENDIAN -std=c11"
