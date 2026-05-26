# Hydrogen math lives in a shared Rust core, not a TypeScript port

A planned web demo (Next.js on Vercel) needs to evaluate the same `|ψ_nlm|²` as the desktop app. We compile `physics.rs` and `volume.rs` to WebAssembly via `wasm-pack` and call them from the browser, rather than porting the math to TypeScript. The desktop app and the web demo stay in lock-step on the wavefunction math — the only thing in this project verified against scipy — instead of risking drift between two implementations.

## Considered Options

- **Port physics + bake to TypeScript.** Smaller bundle, no WASM toolchain, no special HTTP headers. Rejected because every future tweak to the math would have to be mirrored in two languages and re-verified against scipy. The math is the *one* part of this project where correctness is non-obvious and bugs are silent.
- **Pre-bake all orbitals server-side and ship binary volumes.** Trivial client. Rejected because shipping ~84 orbitals × res³×4 bytes (~700MB at res=128) is impractical for a static site, and constrains the demo to a fixed preset list.
- **Evaluate ψ² per-step in the fragment shader.** No CPU work, no WASM. Rejected because it requires rewriting the physics in WGSL/GLSL (i.e. *not* reusing existing code, defeating the goal) and per-sample cost is 10–100× a texture fetch.

## Consequences

- Repo becomes a Cargo workspace: `crates/atom-core` (the wasm-able library — physics, volume, factorial), `crates/atom-desktop` (the current binary, now a thin shell that depends on `atom-core`), `web/` (Next.js app that consumes `atom-core` via wasm-pack).
- `volume.rs` keeps `rayon::par_iter` on native targets; on `wasm32` it falls back to serial `iter` via `#[cfg(not(target_arch = "wasm32"))]`. Threaded WASM (`wasm-bindgen-rayon`) is deferred — the standalone Vercel site can opt in later because it controls its own COOP/COEP headers, but day-one ships single-threaded.
- The shared core stays *minimal* on purpose: only physics + volume bake. Camera math and colormap LUTs are deliberately ported to TypeScript (see ADR-0002 reasoning) — they are trivial, drift-proof, and don't justify FFI overhead.
- The bake runs inside a Web Worker so the main thread stays responsive during ~500ms–2s bake times at the default res=96.
