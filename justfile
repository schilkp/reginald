check:
    cargo test
    ./reginald_codegen/generator_tests/run_tests.bash
    cargo fmt --check
    cargo clippy
    @echo "ALL CHECKS OK"
