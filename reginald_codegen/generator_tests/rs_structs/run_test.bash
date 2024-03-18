#!/bin/bash

# Fail on error:
set -e

echo_green() { printf "\033[0;92m" ; echo "$@" ; printf "\033[0m"; }

# Set CWD to location of this script:
cd "${0%/*}"

# Cleanup/Setup
rm -rf rs_structs_test/src/out*.rs

# Run reginald:
echo "Generating (default)..."
cargo run --quiet --color always -p "reginald_codegen" --    \
    gen -i map.yaml -o rs_structs_test/src/out.rs rs-structs \
    --enum-derive=Debug --enum-derive=PartialEq --struct-derive=Debug

echo "Generating (unpacking error msgs)..."
cargo run --quiet --color always -p "reginald_codegen" --              \
    gen -i map.yaml -o rs_structs_test/src/out_errormsgs.rs rs-structs \
    --unpacking-error-msg=true --enum-derive=Debug --enum-derive=PartialEq --struct-derive=Debug

echo "Generating (no register block mods)..."
cargo run --quiet --color always -p "reginald_codegen" --                   \
    gen -i map.yaml -o rs_structs_test/src/out_noregblockmods.rs rs-structs \
    --register-block-mods=false --enum-derive=Debug --enum-derive=PartialEq --struct-derive=Debug

echo "Generating (no traits)..."
cargo run --quiet --color always -p "reginald_codegen" --                   \
    gen -i map.yaml -o rs_structs_test/src/out_notraits.rs rs-structs \
    --external-traits="crate::out::"

# Compile + Run test exe:
echo "Testing..."
cd rs_structs_test
cargo --color always clippy -- -D warnings
cargo --color always test
echo_green "OK!"
