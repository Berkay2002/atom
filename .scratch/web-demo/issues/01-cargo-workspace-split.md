# Cargo workspace split: extract atom-core from atom-desktop

Status: ready-for-agent

## Parent

`.scratch/web-demo/PRD.md`

## What to build

Convert the single-crate `atom` repo into a Cargo workspace with two members. `crates/atom-core` is a new library crate that owns the wavefunction math and the volume bake — exactly the modules that will later compile to WebAssembly. `crates/atom-desktop` is the existing binary, renamed and relocated, now depending on `atom-core` for physics and volume.

The current development loop must survive unchanged: running `cargo run --release` from the repo root still launches the desktop visualizer, all existing `#[test]` blocks in `physics.rs` and `volume.rs` continue to pass under `cargo test`, and the desktop binary's behavior is byte-for-byte identical to before the split.

`atom-core` contains: physics, volume, and any small helpers those two depend on (e.g. `factorial`). `atom-desktop` contains everything else: render, camera, ui*, colormaps, main, and the shaders directory. The workspace `Cargo.toml` configures `default-members` and `default-run` so the bare `cargo run --release` from the root still works.

This slice does no web work and adds no new behavior. It is a mechanical refactor that unlocks every subsequent slice.

## Acceptance criteria

- [ ] Root `Cargo.toml` is a workspace manifest with `members = ["crates/atom-core", "crates/atom-desktop"]`
- [ ] `crates/atom-core/` exists with `physics.rs`, `volume.rs`, and any necessary helpers; declares `[lib]` with `crate-type = ["rlib"]` (cdylib added in slice 02)
- [ ] `crates/atom-desktop/` exists with the rest of the source tree; its `Cargo.toml` depends on `atom-core` via `path = "../atom-core"`
- [ ] `cargo run --release` from the repo root launches the desktop visualizer with no behavior change
- [ ] `cargo test` from the repo root passes all existing physics and volume tests
- [ ] `cargo build --release` completes with no warnings introduced by the split
- [ ] The `image`, `wgpu`, `winit`, `egui*`, `pollster`, `glam`, `bytemuck` dependencies move to `atom-desktop`; `atom-core` only depends on `rayon` (and what physics needs natively)
- [ ] Existing screenshots, asset paths, and `.gitignore` rules continue to work unchanged

## Blocked by

None — can start immediately
