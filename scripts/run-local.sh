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

if command -v docker >/dev/null 2>&1 \
  && docker image inspect rust:bookworm >/dev/null 2>&1; then
  echo "Cargo Lab: Docker is ready for dependency Fetch and isolated execution."
elif [[ -x /usr/bin/bwrap ]]; then
  echo "Cargo Lab: Bubblewrap detected for offline execution; Fetch still requires Docker."
  echo "Run 'docker pull rust:bookworm' once to enable dependency Fetch." >&2
else
  echo "Cargo Lab: no sandbox backend detected; Cargo actions will stay disabled." >&2
  echo "Install Bubblewrap or start Docker and pull rust:bookworm to enable Cargo execution." >&2
fi

npm --prefix ui run build
exec cargo run --locked -p api
