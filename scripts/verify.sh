#!/usr/bin/env bash
set -euo pipefail

rust_lab_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$rust_lab_root"

npm --prefix ui ci
npm --prefix ui test
npm --prefix ui run build
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
git diff --check
