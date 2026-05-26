# Web demo: standalone browser variant of the orbital visualizer

Status: ready-for-agent

## Problem Statement

The hydrogen orbital visualizer only runs as a native desktop app (`cargo run --release`). Anyone who wants to look at the project — a recruiter, a collaborator, a curious classmate — has to clone the repo, install a Rust toolchain, and build it. That's a high enough barrier that in practice nobody outside the author ever sees the visualization. There is no link the author can paste into a conversation, a portfolio page, or a job application that produces a spinning orbital in the recipient's browser within a few seconds.

## Solution

Ship a standalone Next.js web app, deployed to Vercel at its own URL, that runs the same hydrogen wavefunction visualizer in any modern browser with no install. The wavefunction math (`physics.rs`) and the volume bake (`volume.rs`) are compiled to WebAssembly and reused verbatim from the desktop app — the math has been verified against scipy and must not drift between targets. The browser renders the baked volume with a hand-ported GLSL ray-march shader on raw WebGL2, with a trimmed control panel exposing the parameters a viewer actually cares about (n, l, m, preset, colormap, auto-rotate). The desktop app continues to ship unchanged; the web variant is a parallel demo, not a replacement.

This decision tree is recorded in `docs/adr/0001-shared-rust-core-not-ts-port.md` and `docs/adr/0002-raw-webgl2-not-threejs.md`.

## User Stories

1. As a recruiter clicking a link in a portfolio, I want the visualizer to load and show an orbital within a few seconds, so that I can evaluate the work without installing anything.
2. As a casual viewer, I want to pick from named presets (1s, 2p_x, 3d_z²), so that I can see recognizable orbitals without knowing what the quantum numbers mean.
3. As a physics-curious viewer, I want sliders for n, l, and m, so that I can explore quantum numbers as a continuum rather than a fixed menu.
4. As a viewer changing the principal quantum number, I want l and m sliders to clamp to their valid ranges automatically (l < n, |m| ≤ l), so that I can't request an invalid orbital.
5. As a viewer dragging the n slider quickly from 1 to 6, I want the visualization to keep up with my final value rather than playing back every intermediate orbital, so that scrubbing feels responsive instead of laggy.
6. As a viewer waiting for a bake to finish, I want a visible indication that work is in progress (a shimmer or progress bar over the canvas), so that I know the system saw my input.
7. As a viewer with mouse-and-keyboard, I want to drag on the canvas to orbit the camera and scroll-wheel to zoom, so that I can inspect the orbital from any angle.
8. As a mobile viewer, I want to touch-drag to orbit and pinch to zoom, so that the demo works on a phone.
9. As a viewer who finds the camera disorienting, I want an "auto-rotate" toggle that slowly spins the camera around the orbital, so that I can watch hands-free.
10. As a viewer who likes a particular palette, I want to pick from the same set of colormaps the desktop app offers (inferno, viridis, magma, plasma, grayscale, electron-blue), so that the demo's visual style matches the rest of the project.
11. As a viewer switching from a small orbital (1s) to a large one (4f), I want the camera to automatically reframe to fit the new bounding box, so that the orbital doesn't appear as a tiny dot or get clipped off-screen.
12. As a viewer on a slow laptop or older phone, I want the demo to still render at an interactive frame rate, so that the experience is usable on the kinds of devices my audience actually has.
13. As the author maintaining the project, I want the wavefunction math to live in exactly one place, so that improvements or bug fixes to physics propagate to both the desktop app and the web demo without any manual sync step.
14. As the author shipping a CI build, I want the Vercel build to compile the Rust core to WASM as part of its pipeline, so that I can push to main and have a fresh demo deployed without local build steps.
15. As the author of the desktop app, I want `cargo run --release` from the repo root to keep working exactly as it does today, so that the workspace restructure is transparent to my existing development loop.
16. As the author, I want the standalone site to have its own clean URL (e.g. `atom.vercel.app`), so that I can link to it from my portfolio and from chat without iframes or query-string gymnastics.
17. As a returning viewer, I want the orbital and colormap I last selected to load again when I revisit the page, so that I can come back to a configuration without re-finding it.
18. As an agent porting `raymarch.wgsl` to GLSL, I want the algorithm and accumulation math to remain identical, so that the web variant produces visually indistinguishable output from the desktop renderer for the same parameters.
19. As an agent porting `camera.rs` to TypeScript, I want the orbit/zoom/fit behavior and clamp limits to remain identical, so that camera interactions behave the same as on the desktop.

## Implementation Decisions

### Repository structure

The repo becomes a Cargo workspace with three top-level members:

- `crates/atom-core` — wasm-able library. Owns the wavefunction math (`physics`), the volume bake (`volume`), and a thin `wasm` module exposing the wasm-bindgen surface. This is the single source of truth for the physics.
- `crates/atom-desktop` — the current binary, renamed and relocated. Depends on `atom-core` for physics/volume; keeps `render`, `camera`, `ui*`, `colormaps`, `main`, and `shaders/` as before. `cargo run --release` from the repo root continues to launch the desktop app via a workspace default-run.
- `web/` — Next.js 14+ App Router app (TypeScript). Consumes `atom-core` as a wasm-pack-built package vendored into `web/wasm/`.

### `atom-core::wasm` interface

One exported function — the entire FFI surface:

```
bake(n: u32, l: u32, m: i32, res: usize) -> BakeResult
  where BakeResult = { data: Float32Array (length = res^3), half_extent: f32, peak: f32 }
```

The Float32Array is returned by reference into wasm linear memory (zero-copy from JS's perspective). The browser then uploads it directly into a WebGL2 3D texture. No other Rust symbols are exposed.

### Threading

`volume::bake` keeps `rayon::par_iter` on native targets; on `wasm32` it falls back to plain `iter` gated by `#[cfg(not(target_arch = "wasm32"))]`. The web variant ships single-threaded for v1 so it requires no special HTTP headers (no COOP/COEP, no SharedArrayBuffer). Threaded WASM via `wasm-bindgen-rayon` is a future opt-in for the standalone site only, since Vercel can set the required headers.

### Browser modules

- **`bake-worker`** — Web Worker that loads the WASM and exposes `requestBake({n,l,m,res})` / `cancel()`. Cancellation is implemented as "terminate the worker, spawn a fresh one" — there is no in-WASM checkpointing, so the worker process is the cancellation unit. The bake runs off the main thread so React stays responsive during the ~500ms–2s bake window at the default res=96.
- **`renderer/raymarch`** — Raw WebGL2 (no Three.js, no react-three-fiber). One fullscreen triangle, one fragment program ported from `shaders/raymarch.wgsl` to GLSL ES 3.00, one 3D texture for the volume, one 1D texture for the colormap LUT. Public API: `setVolume`, `setColormap`, `setCamera`, `setParams`, `resize`, `draw`.
- **`camera/orbit-camera`** — TypeScript port of `camera.rs`. Same state (`radius`, `azimuth`, `elevation`, `fovY`, `aspect`), same methods (`orbit`, `zoom`, `fit`, `position`, `viewProj`), same clamp limits (`elevation ∈ ±(π/2 - 0.01)`, `radius ≥ 0.1`), same sensitivity constants. Pure math — no DOM dependencies. Uses `gl-matrix` for `lookAt` and `perspective`.
- **`camera/pointer-input`** — Attaches pointer-down/move/up and wheel handlers (and touch equivalents) to a canvas, translates them into `OrbitCamera.orbit/zoom` calls. Separated from the camera module so the camera stays DOM-free.
- **`hooks/useDebouncedBake`** — React hook encapsulating the debounce-and-cancel state machine. Behavior: when params change, wait 150ms of quiet; if another change arrives in that window, reset the timer. When the timer fires, call `worker.cancel()` then `worker.requestBake(latestParams)`. Returns `{ volume, baking }`.
- **`colormaps`** — TypeScript module with the same six stop arrays as `colormaps.rs`, identical RGB values. Re-derived as a TS constant rather than crossing the WASM boundary (static data, drift-proof).
- **`components/AtomCanvas`** — Owns the canvas, wires `useDebouncedBake` → `renderer` → `requestAnimationFrame` loop. Calls `camera.fit(halfExtent)` on every new volume. Hosts the auto-rotate loop. Calls into `pointer-input` for camera dragging.
- **`components/Controls`** — Renders n (1..6), l (0..n-1), m (-l..l) sliders, preset chips matching the desktop app's preset list, a colormap picker, and an auto-rotate toggle. Enforces the l<n and |m|≤l invariants by clamping when n or l changes.
- **`app/page.tsx`** — Lays out `<AtomCanvas />` and `<Controls />`. Persists the selected params and colormap to `localStorage` and rehydrates on mount.

### UI scope (intentionally trimmed from the desktop)

The web demo exposes: n/l/m sliders, preset chips, colormap picker, auto-rotate toggle. It does **not** expose: resolution (fixed at 96), density coefficient `k` (fixed at 5.0), exposure (fixed at 1.0), fit-to-box button (auto-fit on every bake), screenshot (the browser already does this), HUD visibility (always shown). Default starting orbital matches the desktop: (n=3, l=2, m=1).

### Build pipeline

A `web/scripts/build-wasm.{sh,ps1}` script runs `wasm-pack build crates/atom-core --target web --out-dir ../../web/wasm`. Vercel's build command runs this script before `next build`. Local development uses the same script — no separate dev/prod path.

### Renderer math parity

The ported GLSL shader produces visually indistinguishable output from the desktop WGSL shader for the same `(volume, colormap, k, exposure, steps, camera)`. Ray-march algorithm is unchanged: slab-intersect the bounding box, march N steps from `t_start` to `t_end`, accumulate `density * dt`, apply `intensity = clamp(exposure * (1 - exp(-k * sum)), 0, 1)`, sample LUT, output RGB. The bounding box and texture UVW transform are also unchanged.

## Testing Decisions

A good test in this PRD verifies *external behavior at a module boundary*, not implementation internals. Tests should drive the module's public API and assert on its observable outputs. Mocking the platform (WebGL, Web Worker, the DOM, the network) is explicitly out of scope — the project's existing convention (`CLAUDE.md`) is that renderer and UI code are verified visually.

Three test surfaces are in scope:

1. **`atom-core::wasm` smoke test (Rust, `cargo test --target wasm32-unknown-unknown` or a native cross-check).** One assertion: bake via the wasm-bindgen entry point produces a `data` array equal element-wise (within `1e-6`) to a native `volume::bake` call for a fixed `(n, l, m, res)` like `(2, 1, 0, 32)`. Catches FFI regressions where the bindgen layer accidentally truncates, reorders, or rescales the buffer. Prior art: the existing `bake_produces_unit_peak_after_normalization` and `bake_integral_is_approximately_one` tests in `volume.rs` exercise the same surface natively.

2. **`orbit-camera` (TypeScript, Vitest).** Direct port of the three existing Rust tests in `camera.rs`: `fit` sets radius to twice the half-extent; elevation clamps at the poles under extreme drag; `zoom` multiplies radius. Pure functions, fast, no DOM.

3. **`useDebouncedBake` (TypeScript, Vitest + React Testing Library + fake timers).** The most failure-prone module — debounce and cancellation are easy to get subtly wrong. Tests: (a) a single param change fires exactly one bake after 150ms; (b) two param changes 50ms apart fire exactly one bake, 150ms after the *second* change, with the second params; (c) a param change while a bake is in flight cancels the in-flight bake and starts a new one with the latest params; (d) `baking` is `true` from request start to result delivery. The worker dependency is injected so the test substitutes a fake worker that records `requestBake` / `cancel` calls and resolves on demand.

Out of scope for tests: `bake-worker` (Worker mocking is more work than the bug it would catch), `renderer/raymarch` (mocking WebGL2 same), `colormaps` (constants), `AtomCanvas` and `Controls` (visual verification, per project convention for UI).

## Out of Scope

- **Feature parity with the desktop app.** No screenshot capture, no resolution/k/exposure controls, no HUD visibility toggle, no card collapse, no preset dropdown chrome — the desktop HUD work is a separate codebase concern.
- **Multi-threaded WASM bake.** Single-threaded is sufficient for the demo. `wasm-bindgen-rayon` is a future upgrade for the standalone Vercel deployment only.
- **WebGPU renderer.** Browser support is still partial on Safari and mobile; WebGL2 is the universal target. Reusing `raymarch.wgsl` verbatim via Three.js's WebGPURenderer was considered and rejected (see ADR-0002).
- **Three.js / react-three-fiber.** The render path is one fullscreen triangle; the abstraction is not worth its bundle weight (see ADR-0002).
- **Embedding into another site as a React component or web component.** The demo is standalone-only and linked from the author's portfolio. No iframe route, no published npm package.
- **Server-side prebake of orbitals.** Each bake runs in the visitor's browser on demand; no `.bin` artifacts are committed or hosted.
- **Per-step ψ² evaluation in the fragment shader.** Bake-then-march is preserved exactly as on the desktop.
- **Sharing camera or colormap code via WASM.** Both are deliberately TS-only — they are trivial and drift-proof (see ADR-0001 consequences).
- **Server-side rendering of the canvas.** The Next.js page is a client component; SSR for a canvas-only widget adds no value.
- **Analytics, accounts, persistence beyond localStorage.** Demo only.

## Further Notes

- Default starting orbital is `(n=3, l=2, m=1)` to match `UiState::default()` in `src/ui.rs` and the existing screenshot conventions.
- Default resolution is **96**, not the desktop's 256. Visually nearly identical for these orbitals, ~18× faster to bake, and the 3D texture is ~18× smaller in VRAM — important for mobile.
- The preset list mirrors the desktop's `PRESETS` constant verbatim: 1s, 2s, 2p_x, 2p_y, 2p_z, 3d_xy, 3d_xz, 3d_yz, 3d_(x²-y²), 3d_(z²), 4f_(z³).
- Slider clamping invariants are enforced in `Controls`, not deep in the bake worker — invalid `(n, l, m)` combinations never reach the WASM layer.
- The Rust workspace restructure is mechanical but touches every file's `use` paths. It should be staged as its own commit (or first issue) before any web work begins, so the desktop app's CI continues to pass throughout.
- The author's existing development loop (`cargo run --release` from the repo root) must continue to work after the workspace split. Configure a `default-members` and a `default-run` in the workspace `Cargo.toml` to preserve this.
