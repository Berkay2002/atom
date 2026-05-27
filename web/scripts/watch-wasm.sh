#!/usr/bin/env bash
# Watch crates/atom-core and rebuild the WASM bundle on every change.
#
# Run alongside `next dev` in a side terminal so atom-core edits propagate
# to the browser without manually re-running `npm run wasm`.
#
# Requires cargo-watch: `cargo install cargo-watch`.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
web_dir="$(dirname "$script_dir")"
repo_root="$(dirname "$web_dir")"

if ! command -v cargo-watch >/dev/null 2>&1; then
    echo ""
    echo "cargo-watch not found on PATH."
    echo "Install it with:"
    echo "    cargo install cargo-watch"
    echo ""
    exit 1
fi

cd "$repo_root"
cargo watch \
    --watch crates/atom-core \
    --shell "bash web/scripts/build-wasm.sh"
