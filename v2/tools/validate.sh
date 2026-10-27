#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo fmt --check
rustfmt --edition 2021 --check adapter/mod.rs tools/beta-tests.rs tools/registry-stub.rs tools/tauri-stub.rs
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo +1.85.0 test --locked
cargo check --target x86_64-pc-windows-gnu --all-targets --locked
cargo clippy --target x86_64-pc-windows-gnu --all-targets --locked -- -D warnings
cargo build --example oracle_bridge --locked
node tools/oracle.cjs --compare
node tools/renderer-schema.cjs
python3 tools/materialize-beta.py
cargo test --manifest-path .work/beta-host/Cargo.toml --locked
cargo +1.85.0 test --manifest-path .work/beta-host/Cargo.toml --locked
cargo clippy --manifest-path .work/beta-host/Cargo.toml --all-targets --locked -- -D warnings
cargo check --manifest-path .work/beta-host/Cargo.toml --target x86_64-pc-windows-gnu --all-targets --locked
cargo clippy --manifest-path .work/beta-host/Cargo.toml --target x86_64-pc-windows-gnu --all-targets --locked -- -D warnings
python3 tools/verify-provenance.py
