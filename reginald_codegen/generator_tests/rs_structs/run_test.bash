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
cargo run --quiet --color always -p "reginald_codegen" --                    \
    gen -i ../map.yaml -o rs_structs_test/src/out.rs rs-structs              \
    --enum-derive=Debug --enum-derive=PartialEq                              \
    --struct-derive=Debug --struct-derive=Clone

echo "Generating (external traits)..."
cargo run --quiet --color always -p "reginald_codegen" --                    \
    gen -i ../map.yaml -o rs_structs_test/src/out_ext_traits.rs rs-structs   \
    --external-traits="crate::out::"

echo "Generating (reginald crate traits)..."
cargo run --quiet --color always -p "reginald_codegen" --                    \
    gen -i ../map.yaml -o rs_structs_test/src/out_crate_traits.rs rs-structs \
    --external-traits="reginald::"

echo "Generating (no modules)..."
cargo run --quiet --color always -p "reginald_codegen" --                    \
    gen -i ../map.yaml -o rs_structs_test/src/out_flat.rs rs-structs         \
    --split-into-modules=false

# Compile + Run test exe:
echo "Testing..."
cd rs_structs_test
cargo --color always clippy -- -D warnings
cargo --color always test
echo_green "OK!"
