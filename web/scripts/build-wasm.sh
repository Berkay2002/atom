#!/usr/bin/env bash
# Build the atom-core crate to WebAssembly for the web demo.
#
# Outputs JS shim + .wasm to web/wasm/. Invoked by the Vercel Linux build
# before `next build`. The .ps1 sibling is for local Windows dev.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
web_dir="$(dirname "$script_dir")"
repo_root="$(dirname "$web_dir")"

# On Vercel, neither rustup nor wasm-pack are pre-installed. Install both
# on demand so the project root is build-self-contained.
if ! command -v rustup >/dev/null 2>&1; then
    echo "rustup not found — installing minimal toolchain"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# wasm target is required by wasm-pack but it won't auto-install on Vercel.
rustup target add wasm32-unknown-unknown >/dev/null

if ! command -v wasm-pack >/dev/null 2>&1; then
    echo "wasm-pack not found — installing"
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

cd "$repo_root"
# --out-dir is resolved relative to the crate's manifest dir, so we
# walk back up to the repo root and into web/wasm.
wasm-pack build \
    --target web \
    --out-dir ../../web/wasm \
    crates/atom-core \
    -- --features wasm
