# just manual: https://github.com/casey/just/#readme

_default:
    @just --list

# Format source code
fmt:
    cargo fmt --all

# Run clippy
check:
    cargo clippy --locked --workspace --no-deps --all-targets -- -D warnings --cap-lints warn
    cargo clippy --locked --workspace --no-deps --no-default-features --all-targets -- -D warnings --cap-lints warn

# Fix lint warnings
fix:
    cargo fix --locked --workspace --all-features --all-targets
    cargo clippy --locked --workspace --no-deps --all-features --all-targets --fix

# Run unit tests
test:
    RUST_BACKTRACE=1 cargo test --locked --workspace -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --no-default-features -- --nocapture

# Set up (and update) tooling
setup:
    rustup self update
    cargo install \
        cargo-edit \
        cargo-hack
    pip install -U pre-commit
    pre-commit autoupdate

# Upgrade (and update) dependencies
upgrade:
    RUST_BACKTRACE=1 cargo upgrade --workspace
    cargo update
    cargo minimal-versions check --workspace

# Run pre-commit hooks
pre-commit:
    pre-commit run --all-files
