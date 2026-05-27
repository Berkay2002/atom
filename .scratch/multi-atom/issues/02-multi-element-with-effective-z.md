# Multi-element rendering with Slater's effective Z

Status: ready-for-agent

## Parent

`.scratch/multi-atom/PRD.md`

## What to build

Make the visualizer render orbitals for any of the first 18 elements (H through Ar), with sizes governed by Slater's-rules effective nuclear charge. This is the first user-visible payoff of the multi-atom direction: pick Carbon, see Carbon's 2p; pick Oxygen, see a smaller version.

Three new pieces of physics infrastructure in `atom-core`:

- **`element.rs`** — static table of `ElementData` (symbol, name, atomic_number, electron_config) for H through Ar. Electron configs stored as a machine-readable list of `(n, l, electron_count)` plus a human-readable string for captions. Add smoke tests: 18 entries, atomic numbers 1–18 contiguous, no duplicate symbols.
- **`slater.rs`** — `fn z_eff(element: ElementId, n: u32, l: u32) -> f64` implementing Slater's rules. Internally uses a static screening-contribution table (0.35 / 0.85 / 1.00 contributions per shell distance and orbital type). For orbitals not occupied in the element's ground state, use convention (1) from the spec: compute as if an additional electron were placed in that orbital. Document the convention in the function's doc comment. Add unit tests against textbook values: C-2p ≈ 3.25, F-2p ≈ 5.20, Na-3s ≈ 2.20.
- **`physics.rs`** — thread `z_eff` through the math. `radial(n, l, r)` becomes `radial(n, l, r, z_eff)`. Substitute `ρ = 2·z_eff·r/n` and rescale the normalization factor by `z_eff^(3/2)`. Bare hydrogen is the special case `z_eff = 1.0`. `psi_squared` becomes `psi_squared(orbital, atom_position, z_eff, x, y, z)` — element-agnostic; the caller resolves `z_eff` before invoking. Add scipy cross-check tests at `z_eff ≠ 1` (e.g., He⁺ radial values — 2 or 3 cases).

The bake (`volume::bake_scene` from issue 01) now resolves `z_eff` per atom: for each atom in the scene, look up the element's `z_eff(element, atom.orbital.n, atom.orbital.l)` and pass it to `psi_squared`. The View's `use_bare_z` flag is wired here too: when `true`, override `z_eff` to the bare atomic number `Z` instead of the Slater-shielded value. (The UI toggle that flips the flag arrives in issue 03; the plumbing belongs here.)

Adaptive box sizing accounts for `z_eff` — orbitals shrink as effective charge grows. Reuse or adapt the existing `n,l`-based formula with a `z_eff` scaling factor.

A **functional element picker** on both targets (polish in issue 05):

- **Web:** a simple flat list or basic grid of 18 element symbols; clicking selects. Lives in a sidebar/controls panel alongside the existing `n/l/m` chips. Updates the Scene's atom element and triggers a re-bake.
- **Desktop:** an egui dropdown or compact button strip with the 18 elements. Same wiring.

The UI also clamps invalid `(n, l, m)` combinations per element (e.g., n=1 only allows l=0; the existing chips already do this and continue to). Element switches that change which `(n, l, m)` is valid should pick a sensible default rather than show an empty render.

Web wasm shim and worker protocol updated to carry element + orbital in the Scene.

## Acceptance criteria

- [ ] `atom-core/src/element.rs` exists with 18 entries, smoke tests pass
- [ ] `atom-core/src/slater.rs` exists, textbook-value tests pass (C-2p, F-2p, Na-3s within reasonable tolerance)
- [ ] `physics.rs` accepts `z_eff`; scipy cross-check tests for He⁺ (z_eff=2) pass
- [ ] `bake_scene` resolves `z_eff` per atom from the element + orbital; when `view.use_bare_z = true`, uses bare `Z` instead
- [ ] Desktop: picker present; selecting Carbon and `(n=2, l=1, m=0)` renders a visibly smaller 2p cloud than Hydrogen's `(n=2, l=1, m=0)`
- [ ] Web: picker present; same Carbon-vs-Hydrogen visual difference in the browser
- [ ] Invalid `(n, l, m)` combinations after element switch are auto-corrected to a sensible default rather than crashing or rendering empty
- [ ] WASM build succeeds; bake worker handles the new Scene payload
- [ ] All existing tests still pass

## Blocked by

`.scratch/multi-atom/issues/01-scene-shaped-bake-api.md`
