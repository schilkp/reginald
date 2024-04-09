check:
    cargo fmt --check
    cargo clippy
    ./reginald_codegen/generator_tests/run_tests.bash
    echo "ALL CHECKS OK"
