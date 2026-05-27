# WASM dev-loop: cargo-watch chore

Status: ready-for-agent

## Parent

`.scratch/multi-atom/PRD.md`

## What to build

Close the stale-WASM footgun described in the spec's "Per-target boundary" section: when `next dev` is running and a developer edits `crates/atom-core/**/*.rs`, the change does not reach the browser until `npm run wasm` is re-run manually.

Add a small dev-watch script that watches `crates/atom-core` and re-runs `web/scripts/build-wasm.ps1` on every change. Live alongside the existing scripts, not as a replacement for any of them.

Concretely:

- Add a script at `web/scripts/watch-wasm.ps1` (PowerShell, matching project convention) that invokes `cargo watch -w ../crates/atom-core -s "pwsh ./scripts/build-wasm.ps1"` (paths adjusted as needed for the actual cwd).
- Add an npm script entry: `"watch:wasm": "pwsh ./scripts/watch-wasm.ps1"`.
- Document the workflow in `web/CLAUDE.md` (or the closest existing doc): "while `next dev` is running, also run `npm run watch:wasm` in a side terminal so atom-core edits propagate to the browser."
- If `cargo-watch` is not installed, the script should print a friendly install hint (`cargo install cargo-watch`) and exit non-zero rather than failing silently.

This slice is a pure developer-experience chore. It does not affect production builds or CI.

## Acceptance criteria

- [ ] `web/scripts/watch-wasm.ps1` exists and watches `crates/atom-core` via `cargo watch`
- [ ] `npm run watch:wasm` works from the `web/` directory
- [ ] When `cargo-watch` is missing, the script prints an actionable install hint and exits non-zero
- [ ] `web/CLAUDE.md` documents the side-terminal pattern for use during `next dev`
- [ ] No change to `npm run dev`, `npm run wasm`, or `npm run build` behavior

## Blocked by

None — can start immediately
