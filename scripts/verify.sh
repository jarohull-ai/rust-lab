#!/usr/bin/env bash
set -euo pipefail

rust_lab_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$rust_lab_root"
export PATH="$rust_lab_root/ui/node_modules/.bin:$PATH"

if [ ! -d "ui/node_modules" ]; then
  npm --prefix ui ci
fi
npm --prefix ui test
npm --prefix ui run build
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
if rg -n 'host-unsandboxed|popen\(|system\(' crates/api/src/workspace.rs crates/runtime/src/codegen_cpp.rs; then
  echo "Unsafe host fallback or shell execution primitive detected." >&2
  exit 1
fi
if rg -n 'uses: [^#[:space:]]+@(v[0-9]+|stable|main|master)([[:space:]]|$)' .github/workflows; then
  echo "GitHub Actions must be pinned to immutable commit SHAs." >&2
  exit 1
fi
rg -q 'let mut p: \*mut libc::c_char' vendor/glib-0.18.5-rustlab/src/variant_iter.rs
rg -q '&mut p,' vendor/glib-0.18.5-rustlab/src/variant_iter.rs
if ! rg -q '^  "version": "[0-9]+\.[0-9]+\.[0-9]+",$' src-tauri/tauri.conf.json; then
  echo "Tauri installer version must be numeric MAJOR.MINOR.PATCH for MSI/RPM compatibility." >&2
  exit 1
fi
rg -q '^  prepare-release:$' .github/workflows/release.yml
rg -q 'releaseId: \$\{\{ needs\.prepare-release\.outputs\.release_id \}\}' .github/workflows/release.yml
if rg -n '^          tagName:' .github/workflows/release.yml; then
  echo "Matrix jobs must upload through the prepared release ID instead of racing to create a tag release." >&2
  exit 1
fi
git diff --check
