# Scene projection bake

Status: ready-for-agent

## Parent

PRD: `.scratch/wasm-scene-projection/PRD.md`

## What to build

Move the browser bake path to the same Scene projection concept used by the codec path. The web worker protocol may remain optimized for the single-Atom slice, but its shape should mirror Scene language and the WASM export should avoid adding another independent tuple interface.

## Acceptance criteria

- [ ] Browser bake requests still produce the same normalized volume data, resolution, half-extent, and peak for representative Scenes.
- [ ] The worker protocol mirrors Scene, Atom, Orbital, and View language.
- [ ] The WASM data ownership contract remains explicit: JS copies or uploads the returned view before the Rust result is dropped.
- [ ] Existing bake cancellation behavior is preserved.
- [ ] WASM parity tests still compare projection bake output with direct core bake output.

## Blocked by

- `.scratch/wasm-scene-projection/issues/01-scene-projection-codec.md`
