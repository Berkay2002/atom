# Scene-shaped bake API (refactor, no behavior change)

Status: ready-for-agent

## Parent

`.scratch/multi-atom/PRD.md`

## What to build

Introduce the `Scene`-of-`Atom`s data model in `atom-core` and reshape the volume bake to consume it. No user-visible change; this slice is pure plumbing that commits to the Approach-3 architecture so every subsequent slice has a clean foundation.

Add three new modules in `atom-core`:

- `scene.rs` — `Scene { atoms: Vec<Atom>, view: View }`, `Atom { element: ElementId, position: [f64; 3], orbital: Orbital }`, `Orbital { n: u32, l: u32, m: i32 }`, `View { use_bare_z: bool, camera: CameraState, colormap: ColormapId, exposure: f32 }`. Per the spec, `Orbital` is a single triple (not a `Vec`) and `use_bare_z` lives on `View` (not on `Atom`).
- A minimal `ElementId` for now (just a `u32` atomic number, or a tiny enum with only `Hydrogen`). Full element table arrives in issue 02.
- Re-export the new types from `lib.rs`.

Replace `volume::bake(n, l, m, res)` with `volume::bake_scene(scene: &Scene, res: u32) -> Volume`. Internally:

- Adaptive box: `half_extent = max over scene.atoms of (|atom.position| + atom_radius(orbital))`. For slice 1, single atom at origin collapses to today's behavior.
- Per-voxel evaluation iterates over `scene.atoms` and sums densities (`Σ |ψ_atom|²`).
- Peak-normalize across the whole volume, not per-atom.

Update both call sites:

- Desktop (`atom-desktop/src/main.rs` and wherever the bake is triggered): construct a single-atom `Scene` and call `bake_scene`.
- Web wasm shim (`atom-core/src/wasm.rs`) and the worker (`web/src/lib/bake-worker.ts`): the JS-side API becomes `bake_scene(sceneJson, res)` or an equivalent thin signature. Update the protocol type in `bake-worker.ts` and `bake/client.ts` accordingly.

Hydrogen-only at this stage. `z_eff` is hard-coded to `1.0`. The new `View` fields (`use_bare_z`, etc.) exist in the struct but no UI is wired to them yet — that's slices 03 and onwards.

## Acceptance criteria

- [ ] `crates/atom-core/src/scene.rs` exists with the structs from the spec (Scene/Atom/Orbital/View)
- [ ] `volume::bake_scene(&Scene, u32) -> Volume` replaces `volume::bake`
- [ ] Desktop builds and runs; selecting an orbital from the UI still produces the same visual as before this slice
- [ ] Web demo builds (`npm run build` succeeds) and the in-browser bake worker still produces a working orbital render
- [ ] **Regression test** in `volume`: `bake_scene(Scene { single H atom at origin, Orbital { n: 2, l: 1, m: 0 } }, res = 32)` produces voxel-for-voxel identical output to the old `bake(2, 1, 0, 32)` — capture the old output as a fixture if needed
- [ ] All existing `physics.rs` and `volume.rs` tests still pass
- [ ] `cargo build --release` completes with no new warnings
- [ ] WASM rebuild (`npm run wasm`) succeeds and `web/wasm/atom_core.js` exports the new API surface

## Blocked by

None — can start immediately
