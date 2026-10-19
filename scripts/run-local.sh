#!/usr/bin/env bash
set -euo pipefail

rust_lab_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$rust_lab_root"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust/Cargo is required to run Rust Lab." >&2
  exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "Node.js/npm is required to build the Rust Lab interface." >&2
  exit 1
fi

if [[ ! -d ui/node_modules ]]; then
  npm --prefix ui ci
fi

npm --prefix ui run build
exec cargo run -p api
