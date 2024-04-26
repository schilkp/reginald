check:
    cargo test --workspace
    ./reginald_codegen/generator_tests/run_tests.bash
    cargo fmt --check
    cargo clippy --all-features --workspace
    @echo "ALL CHECKS OK"

tokei:
    tokei --exclude Unity
