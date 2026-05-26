# Tracer bullet: 1s orbital renders in a browser on a Vercel URL

Status: ready-for-agent

## What to build

The thinnest possible end-to-end slice that proves the entire stack works: a visitor opens a Vercel-deployed URL in any modern browser and sees a hydrogen 1s orbital ray-marched in their own GPU. The Rust math runs in the browser via WebAssembly; the volume is baked off-thread in a Web Worker; the renderer is raw WebGL2 with a hand-ported GLSL shader.

To stay thin, everything except the rendering path is hardcoded:

- Orbital is fixed at `(n=1, l=0, m=0)`. No sliders, no presets, no React state for params.
- Camera is fixed at one view (e.g. azimuth=45°, elevation=30°, radius=2·half_extent). No drag, no scroll, no auto-rotate.
- Colormap is a single grayscale gradient inlined directly in the fragment shader. No 1D LUT texture yet.
- No controls UI. The page is just a `<canvas>` filling the viewport.
- No persistence, no shimmer/loading state.

What this slice *does* deliver, end-to-end:

- `web/` directory with a Next.js 14+ App Router app (TypeScript), client-only.
- `web/scripts/build-wasm.{sh,ps1}` script that runs `wasm-pack build crates/atom-core --target web --out-dir ../../web/wasm` and is invoked by the Vercel build before `next build`.
- `bake-worker.ts` Web Worker that loads the WASM module and exposes a single `bake(n,l,m,res)` message. Cancellation is not yet implemented (no need at this slice — there's no input).
- `renderer/raymarch.ts` raw WebGL2 module: fullscreen-triangle vertex shader + GLSL ES 3.00 fragment shader ported from `shaders/raymarch.wgsl`. Same slab-intersect + accumulate + exposure math. Public API: `setVolume`, `setCamera`, `setParams`, `resize`, `draw`. The colormap path can be stubbed (hardcoded grayscale lookup inside the shader) until slice 06.
- `components/AtomCanvas.tsx` wires the worker → renderer → requestAnimationFrame loop with the hardcoded params.
- `app/page.tsx` renders `<AtomCanvas />`.
- Vercel deployment of the standalone site at its own URL.

The 1s orbital must look visually correct (recognizable spherical density centered at the origin, smoothly falling off — not a flat box, not a black screen, not banding).

## Acceptance criteria

- [ ] `web/` scaffold exists with Next.js 14+, TypeScript, App Router
- [ ] `web/scripts/build-wasm.{sh,ps1}` builds `atom-core` to WASM and places artifacts in `web/wasm/`
- [ ] Vercel build configuration invokes the WASM build script before `next build`
- [ ] `bake-worker.ts` loads the WASM and answers a `bake(1, 0, 0, 96)` request with a `data` Float32Array, `half_extent`, and `peak`
- [ ] `renderer/raymarch.ts` compiles a GLSL ES 3.00 fragment shader ported from `raymarch.wgsl` and renders a fullscreen triangle that samples a 3D texture
- [ ] The deployed Vercel URL renders a recognizable 1s orbital (smooth, centered, monotonically decreasing density from the origin)
- [ ] The render keeps a stable framerate on a modern desktop browser (no per-frame stalls; rAF loop is clean)
- [ ] No special HTTP headers are required (no COOP/COEP)
- [ ] The Vercel deployment URL is recorded in the issue comments or PR description for downstream slices to reference

## Blocked by

- Issue 02 (atom-core::wasm bindings)
