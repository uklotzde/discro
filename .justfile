# SPDX-FileCopyrightText: The discro authors
# SPDX-License-Identifier: CC0-1.0

# just manual: https://github.com/casey/just/#readme

_default:
    @just --list

# Format source code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --locked --workspace --no-deps --all-targets -- -D warnings --cap-lints warn
    cargo clippy --locked --workspace --no-deps --all-targets --no-default-features -- -D warnings --cap-lints warn
    cargo clippy --locked --workspace --no-deps --all-targets --features tokio -- -D warnings --cap-lints warn
    cargo clippy --locked --workspace --no-deps --all-targets --no-default-features --features tokio -- -D warnings --cap-lints warn

# Run unit tests
test:
    RUST_BACKTRACE=1 cargo test --locked --workspace -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --no-default-features -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --features tokio -- --nocapture
    RUST_BACKTRACE=1 cargo test --locked --workspace --no-default-features --features tokio -- --nocapture

# Set up (and update) tooling
setup:
    # Ignore rustup failures, because not everyone might use it
    rustup self update || true
    # cargo-edit is needed for `cargo upgrade`
    cargo install cargo-edit just
    pip install -U pre-commit
    #pre-commit install --hook-type commit-msg --hook-type pre-commit

# Upgrade (and update) dependencies
upgrade: setup
    pre-commit autoupdate
    cargo upgrade
    cargo update

# Run pre-commit hooks
pre-commit:
    pre-commit run --all-files
