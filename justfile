full_check:
    cargo test --workspace
    REGINALD_TEST_C_FUNCPACK_ADDITIONAL_COMPILERS=arm-none-eabi-gcc cargo test --workspace -- --ignored
    cargo fmt --check
    cargo clippy --all-features --workspace
    @echo "ALL CHECKS OK"

check:
    cargo test --workspace
    cargo fmt --check
    cargo clippy --all-features --workspace
    @echo "ALL CHECKS OK"

tokei:
    tokei --exclude Unity

cov:
    cargo tarpaulin --all-targets --doc --out Html -- --include-ignored
