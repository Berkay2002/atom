# Multi-atom direction (slice 1) — design

**Status:** approved direction, slice 1 not yet implemented
**Date:** 2026-05-27
**Supersedes:** none (extends the hydrogen-only scope assumed in ADR-0001)

## Why

`atom` today visualizes only hydrogen `|ψ_nlm|²`. The math in `crates/atom-core/src/physics.rs` is hardcoded for `Z = 1` (no nuclear-charge parameter, no shielding). The closed-form `R_nl` exists only because hydrogen's one-electron Schrödinger equation separates analytically.

We want the app to support multiple elements and, eventually, molecular bonding — turning a single-orbital sandbox into an interactive learning tool for periodic-table-scale chemistry.

## Long-arc direction

Four possible meanings of "mix and match" were considered:

- **A — Compare elements** (C 2p vs O 2p vs F 2p side-by-side)
- **B — Compose orbitals within one atom** (electron-config game, sum of `|ψ_nlm|²`)
- **C — Bond atoms into molecules** (LCAO — drag two H's together, form H₂)
- **D — Free QM sandbox** (arbitrary superpositions; element identity barely matters)

**Chosen arc:** A first (cheapest extension, viscerally useful), C as the eventual destination (chemistry actually lives in bonding). B and D are not on the current trajectory.

## Locked decisions

1. **Physics model:** Tier 2 — hydrogen-like math with **Slater's-rules effective nuclear charge** (`Z_eff`). Tier 1 (bare `Z`) remains accessible as a **UI toggle** so users can see shielding's effect by comparison. Tier 3 (STO) deferred.
2. **Shape:** Sandbox + Guided Tours. No quiz/levels mode. Tours are JSON content stepped through prev/next.
3. **Architecture:** **Approach 3 staged.** Data model from day one is `Scene = [Atom { element, position, orbital }]`. Slice 1 only ever instantiates a single atom at the origin, but the shape generalizes to side-by-side comparison and to molecular bonding without a second redesign.
4. **State serialization:** `Scene` round-trips through a versioned URL-encodable string so tour steps and user states are shareable.

## Slice 1 scope

### In scope
- `atom-core` gains element data (H–Ar), Slater's-rules `Z_eff` function, and `Scene`/`Atom`/`Orbital`/`View` types.
- Physics signature becomes `psi_squared(orbital, atom_position, z_eff, x, y, z)`. Same closed-form math, parameterized by `z_eff`.
- Volume bake becomes `bake_scene(scene, res) -> Volume`. Adaptive box spans all atoms; per-voxel evaluation sums **densities** across atoms (`Σ |ψ_atom|²`).
- UI: periodic-table-shaped picker (18 cells, periods 1–3), existing `n/l/m` chips, "Bare Z / Effective Z" toggle, a "what am I looking at" caption pulled from a small per-orbital table.
- 2–3 starter tours (e.g., "Shielding across period 2") authored as JSON in `web/public/tours/`.
- `Scene` ↔ URL encoder/decoder in `atom-core` so desktop and web produce identical strings.
- Watch-script chore: `cargo-watch -w crates/atom-core -s "pwsh web/scripts/build-wasm.ps1"` documented in `web/CLAUDE.md` to avoid stale-WASM footgun during dev.

### Out of slice 1 (deferred)
- Side-by-side multi-atom comparison rendering (data model is ready; GPU and UI work deferred).
- Molecular bonding (LCAO) — the long-arc C destination.
- d/f orbital handling with full Slater (transition metals, lanthanides).
- Tier 3 STOs.
- Tour authoring UI.
- Quiz/levels mode.
- Time evolution.
- Animated morph between tour steps (slice 1 = hard cut).

### Success criteria
A visitor lands on the URL, sees neon's 2p, clicks "Bare Z" and watches the orbital collapse inward, then clicks "Next" through "Shielding across period 2" walking B → C → N → O → F at the same orbital.

## Data model

```rust
// atom-core::element
struct ElementData {
    symbol: &'static str,        // "C"
    name: &'static str,          // "Carbon"
    atomic_number: u32,          // 6
    electron_config: &'static str, // "[He] 2s² 2p²" — for captions
}

// atom-core::slater
fn z_eff(element: ElementId, n: u32, l: u32) -> f64;
// Walks the element's electron config, sums screening contributions
// from a static (0.35 / 0.85 / 1.00) table per Slater's rules.

// atom-core::scene
struct Orbital { n: u32, l: u32, m: i32 } // single triple — NOT Vec
struct Atom {
    element: ElementId,
    position: [f64; 3],   // a₀; slice 1 always [0, 0, 0]
    orbital: Orbital,
}
struct View {
    use_bare_z: bool,        // global, NOT per-atom
    camera: CameraState,
    colormap: ColormapId,
    exposure: f32,
}
struct Scene { atoms: Vec<Atom>, view: View }
```

### Why `Orbital` is a single triple, not `Vec`
A true superposition density is `|Σ c·ψ|²`, not `Σ |c·ψ|²`. Pre-shaping `Orbital` as a `Vec` would hide that physics decision and tempt a wrong implementation later. When option B (in-atom composition) is added, the structure changes deliberately.

### Why `use_bare_z` lives in `View`, not `Atom`
A scene showing "Carbon with bare Z next to Oxygen with effective Z" is pedagogically nonsense. Bare-vs-effective is a global "which lesson are we teaching" toggle. Treating it as a view setting keeps that constraint structural.

### Unoccupied-orbital convention
For orbitals not occupied in the element's ground state (e.g., Carbon's 4f), `z_eff` is computed as if **an additional electron were placed in that orbital**. Documented in the `z_eff` doc-comment.

## Physics and bake changes

**Physics:** thread `z_eff` through. `radial` becomes `radial(n, l, r, z_eff)` with `ρ = 2·z_eff·r/n` and the normalization rescaled by `z_eff^(3/2)` (standard textbook result). Bare hydrogen is the special case `z_eff = 1.0`.

**Separation of concerns:** `atom-core::physics` knows nothing about elements or shielding. Callers resolve `z_eff` from `(element, n, l, use_bare_z)` before invoking physics. Keeps the scipy verification path intact.

**Bake:** `bake_scene(scene, res)`:
1. **Adaptive box** spans all atoms: `half_extent = max(|atom.position| + atom_radius(n, l, z_eff))` over the scene. Collapses to current behavior for a single atom at origin.
2. **Per-voxel** loops atoms and sums densities (`Σ |ψ_atom|²`). For slice 1, one iteration.
3. **Peak normalization** is per-scene, not per-atom.

Rayon parallelism on the outer voxel loop is unchanged.

## Per-target boundary

### Lives in `atom-core` (propagates to both desktop and web)
- `physics.rs` — math + `z_eff` parameter
- `element.rs` — periodic-table data, electron configs
- `slater.rs` — `z_eff()` function + screening constants
- `scene.rs` — `Scene/Atom/Orbital/View` structs + serde encode/decode
- `volume.rs` — `bake_scene`

### Stays per-target (intentionally duplicated)
| Concern | Desktop | Web | Why |
|---|---|---|---|
| Camera math | `atom-desktop/src/camera.rs` | `web/src/lib/camera/` | ADR-0001 carve-out: trivial, drift-proof, FFI overhead not worth it. |
| Colormap LUTs | `atom-desktop/src/colormaps.rs` | `web/src/lib/colormaps.ts` | Static constants. |
| Renderer | `atom-desktop/src/render.rs` (wgpu) | `web/src/lib/renderer/` (WebGL2) | Different graphics APIs by design (ADR-0002). |
| UI chrome | egui | React/Next.js | Native vs. web. |
| Tour content | (none in slice 1) | `web/public/tours/*.json` | Tours are web-demo content. |
| URL routing glue | (n/a) | `web/src/lib/scene-url.ts` | Routing is web-only; the *encoding* is shared via `atom-core`. |

### Propagation mechanics
- **Desktop:** `cargo run --release` re-links `atom-core` automatically.
- **Web:** `wasm-pack build` regenerates `web/wasm/atom_core.{js,wasm}`. Runs on `npm run dev` (via `predev`) and `npm run build`, **not** on file save while `next dev` is already running. The watch-script chore (above) closes this gap.

### Why `scene.rs` earns its `atom-core` slot
The Scene struct definitions could trivially be per-target. The **encoder/decoder** cannot — if desktop and web produce or accept different URL strings for the same Scene, shareable URLs silently break across clients. The lock-step argument for shared serde is the same lock-step argument for shared physics.

### Pressure on ADR-0001
Adding three new modules to `atom-core` pushes against the "stays minimal" wording. Each is justified on lock-step grounds (same `z_eff`, same Scene encoding). If `atom-core` keeps growing past this slice, an ADR-0001 amendment is warranted.

## Testing

Follows the project's existing asymmetry (physics unit-tested against scipy; visual code verified by eye). Per the project's `CLAUDE.md`, no invented unit tests for the renderer/UI.

### New unit tests in `atom-core`
- `physics`: `radial(n, l, r, z_eff=2.0)` cross-checked against scipy for He⁺. Two or three cases.
- `slater`: textbook values — C-2p ≈ 3.25, F-2p ≈ 5.20, Na-3s ≈ 2.20.
- `scene`: round-trip serde (`Scene → string → Scene` equality) **and** a fixed-string fidelity test (`"v1:..."` decodes to expected `Scene`). The fidelity test is what prevents URL format drift between desktop and web.
- `element`: smoke (18 elements, atomic numbers 1–18 contiguous, no duplicate symbols).

### Regression test
`bake_scene` with a single H atom at origin (`n=2, l=1, m=0`) must produce voxel-for-voxel identical output to today's `bake(2, 1, 0, res=32)`. Locks in that Approach 3 is a true generalization, not a behavioral change.

### Visual verification (per CLAUDE.md)
Renderer, picker layout, tour stepper UX, "does Carbon's 2p actually look right" eyeballed against textbook images.

## Open questions (decide at implementation time)

- Exact URL encoding syntax. Versioning is committed (`v1:` prefix); format isn't.
- Periodic-table picker visual polish. Functional 18-cell grid first; aesthetic pass later.
- Whether desktop ships the element picker in slice 1 or stays hydrogen-only. Leaning: ship it on desktop too, since the physics goes through `atom-core` regardless and a desktop dev would want to dogfood the new orbitals.

## Known design tensions accepted

- **Slater's rules are an approximation.** Orbital sizes are roughly right, not chemistry-grade. The Bare-Z toggle exposes the trade-off rather than hiding it.
- **`atom-core` growth.** Three new modules pressure ADR-0001's minimalism. Accepting the pressure here; revisiting if growth continues.
