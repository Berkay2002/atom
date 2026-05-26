# atom-core::wasm bindings + parity smoke test

Status: ready-for-agent

## Parent

`.scratch/web-demo/PRD.md`

## What to build

Add a `wasm` module to `atom-core` that exposes the volume bake as a single wasm-bindgen entry point. From the browser's perspective, the entire Rust API surface is one function:

```
bake(n: u32, l: u32, m: i32, res: usize) -> BakeResult
  where BakeResult = { data: Float32Array (length = res^3), half_extent: f32, peak: f32 }
```

The `data` field is returned as a view into wasm linear memory (no copy on the JS side). `atom-core` gains `crate-type = ["cdylib", "rlib"]` so it can be both linked by `atom-desktop` (rlib) and compiled to `.wasm` (cdylib). Add a `wasm` feature on `atom-core` that gates the wasm-bindgen dependency and the `wasm` module — desktop builds don't pull in wasm-bindgen.

`volume::bake` becomes platform-aware: on native targets it continues to use `rayon::par_iter`; on `wasm32` it falls back to plain serial `iter`, gated by `#[cfg(not(target_arch = "wasm32"))]`. The desktop app's bake performance is unaffected.

Verify the bindings by writing a smoke test that confirms the WASM bake output equals the native bake output element-wise within `1e-6` for a fixed `(n=2, l=1, m=0, res=32)`. Run it via `wasm-pack test --node` (or equivalent) as part of `cargo test` for the workspace.

No web app yet — this slice produces a buildable `.wasm` artifact and a passing parity test.

## Acceptance criteria

- [ ] `crates/atom-core/Cargo.toml` declares `crate-type = ["cdylib", "rlib"]` and a `wasm` feature gating `wasm-bindgen`
- [ ] `atom-core::wasm::bake(n, l, m, res)` is exposed to JS via `#[wasm_bindgen]`
- [ ] Return value carries `data: Float32Array`, `half_extent: f32`, `peak: f32`; the JS-visible `data` is a memory view, not a copy
- [ ] `volume::bake` compiles on both native and `wasm32-unknown-unknown` targets; native build still uses rayon
- [ ] `wasm-pack build crates/atom-core --target web` succeeds and produces a `.wasm` + JS shim
- [ ] A smoke test asserts WASM bake == native bake within `1e-6` element-wise for `(2, 1, 0, 32)`
- [ ] Desktop `cargo run --release` continues to work unchanged

## Blocked by

- Issue 01 (Cargo workspace split)
