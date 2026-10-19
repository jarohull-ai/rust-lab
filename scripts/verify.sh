#!/usr/bin/env bash
set -euo pipefail

rust_lab_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$rust_lab_root"

npm --prefix ui ci
npm --prefix ui test
npm --prefix ui run build
cargo test --workspace
cargo clippy --all-targets --all-features -- -D warnings
