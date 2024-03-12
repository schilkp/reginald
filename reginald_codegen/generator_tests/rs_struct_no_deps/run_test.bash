#!/bin/bash

# Fail on error:
set -e

echo_green() { printf "\033[0;92m" ; echo "$@" ; printf "\033[0m"; }

# Set CWD to location of this script:
cd "${0%/*}"

# Cleanup/Setup
rm -rf rs_struct_no_deps_test/src/out.rs

# Run reginald:
echo "Generating (default)..."
cargo run --quiet --color always -p "reginald_codegen" --  \
    gen -i map.yaml -o rs_struct_no_deps_test/src/out.rs rs-struct-no-deps \
    --enum-derive=Debug --enum-derive=PartialEq --struct-derive=Debug

echo "Generating (unpacking error msgs)..."
cargo run --quiet --color always -p "reginald_codegen" --            \
    gen -i map.yaml -o rs_struct_no_deps_test/src/out_errormsgs.rs rs-struct-no-deps \
    --unpacking-error-msg=true --enum-derive=Debug --enum-derive=PartialEq --struct-derive=Debug

echo "Generating (no register block mods)..."
cargo run --quiet --color always -p "reginald_codegen" --                 \
    gen -i map.yaml -o rs_struct_no_deps_test/src/out_noregblockmods.rs rs-struct-no-deps \
    --register-block-mods=false --enum-derive=Debug --enum-derive=PartialEq --struct-derive=Debug

# Compile + Run test exe:
echo "Testing..."
cd rs_struct_no_deps_test
cargo --color always test
echo_green "OK!"
