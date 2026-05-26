# Web demo renders with raw WebGL2, not Three.js

The web variant uses a single `<canvas>` with a raw WebGL2 program and a hand-ported GLSL fragment shader (translated from `shaders/raymarch.wgsl`). It does not use Three.js, react-three-fiber, or a Three.js-based abstraction. The rendering work is one fullscreen triangle that samples a 3D texture and a 1D LUT — there is no scene graph, no geometry, no lighting, no materials, no post-processing. Three.js would add ~600KB of dependencies whose features the app never invokes.

## Considered Options

- **react-three-fiber + custom ShaderMaterial.** Nice JSX ergonomics, OrbitControls in one line. Rejected because the ergonomic win is small (the entire render loop is ~30 lines of WebGL2) and the bundle cost is large for a demo whose value-prop includes "loads fast on a portfolio link click."
- **Three.js WebGPURenderer reusing `raymarch.wgsl` verbatim.** Maximum shader reuse. Rejected because WebGPU support on Safari/mobile is still partial, so a WebGL2 fallback would be needed anyway — at which point the WGSL-verbatim reuse evaporates.
- **Raw WebGPU (no Three.js), reusing `raymarch.wgsl`.** Closest to the desktop pipeline structurally. Rejected for the same WebGPU-support reason; the demo is meant to work for any portfolio visitor.

## Consequences

- `raymarch.wgsl` is hand-ported to GLSL ES 3.00 in `web/src/shaders/raymarch.frag` (and a fullscreen-triangle vertex shader). The algorithm is unchanged: front-to-back ray-march through a 3D texture, accumulate density, exposure + LUT lookup at the end. The port is mechanical (`texture_3d` → `sampler3D`, `vec4<f32>` → `vec4`, `textureSampleLevel` → `texture`) and short enough (~80 lines) to keep in sync by hand if either shader is edited.
- Camera input handling (pointer drag → azimuth/elevation, wheel → zoom) is written fresh in TypeScript using `gl-matrix` for matrix math. The orbit-camera math in `src/camera.rs` is not shared; it is small enough that a TS rewrite is cheaper than the FFI overhead of calling it from WASM each frame.
- The colormap stop tables in `src/colormaps.rs` are also duplicated as a TS module rather than exported from `atom-core`. They are static constants and cannot meaningfully drift.
- Bundle stays small: React + a thin WebGL2 helper + the wasm-bindgen glue + the `atom-core` WASM blob. No 3D-engine dependency.
