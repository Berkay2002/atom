<!-- BEGIN:nextjs-agent-rules -->
# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` before writing any code. Heed deprecation notices.
<!-- END:nextjs-agent-rules -->

## Dev loop with live WASM rebuilds

`next dev` only watches JS/TSX. Edits under `crates/atom-core/**/*.rs` do
not reach the browser until `npm run wasm` runs again. To close that gap,
run the watcher in a side terminal next to `next dev`:

```powershell
cd web
npm run dev          # terminal A — Next.js dev server
npm run watch:wasm   # terminal B — rebuilds wasm on atom-core changes
```

`watch:wasm` requires `cargo-watch` (install once: `cargo install cargo-watch`).
If `cargo-watch` is missing, the script prints an install hint and exits
non-zero. This is dev-only — `npm run dev`, `npm run wasm`, and
`npm run build` are unchanged.
