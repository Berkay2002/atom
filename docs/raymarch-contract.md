# Ray-march contract

This document names the renderer semantics that must stay aligned between the desktop WGSL path and the web GLSL path.

It is a contract for behavior, not a shared renderer abstraction. Desktop stays on `wgpu` + WGSL, web stays on raw WebGL2 + GLSL, per ADR-0002.

## Volume data

- The baked volume is peak-normalized scalar density data in a cubic `res^3` grid, so render inputs are in `[0, 1]`.
- Storage is row-major with `x` changing fastest, then `y`, then `z`.
- The grid represents a cube centered on the origin with half-extent `half_extent`.
- World-space sample positions are mapped into texture coordinates with `uvw = (p + half_extent) / (2 * half_extent)` while `p` is inside the cube.

## Camera inputs

- The ray-marcher receives camera world position.
- The ray-marcher receives the inverse view-projection matrix.
- Rays are reconstructed by unprojecting the pixel's NDC position through that inverse matrix and pairing it with the world-space camera origin.

## Ray-box intersection

- The volume is intersected as an axis-aligned cube `[-half_extent, +half_extent]^3`.
- Slab intersection determines entry and exit distances.
- Pixels that miss the cube return black.

## Marching

- March count is a fixed positive integer supplied by renderer parameters.
- Desktop currently passes the uploaded volume resolution as `steps`.
- Web currently uses an explicit renderer parameter (`steps = 256`) while its interactive bake resolution is lower; that is intentional renderer tuning, not a data-layout change.
- Step length is `(t_exit - t_entry) / steps`.
- Sampling uses midpoint steps along the segment from entry to exit.

## Accumulation and tone mapping

- At each step, accumulate `sum += density * step_length`.
- The final scalar intensity is `clamp(exposure * (1 - exp(-k * sum)), 0, 1)`.
- The `k` parameter is the absorption coefficient.
- The exposure parameter scales the post-saturation output before LUT lookup.

## LUT sampling

- The final intensity indexes a 256-entry 1D LUT.
- LUT interpolation is linear between author-supplied color stops, then linear-filtered at sample time.
- Desktop uses a `texture_1d<f32>` binding; web uses a `sampler2D` 256x1 texture with the same stop interpolation and `v = 0.5` sampling convention.
- LUT wrap mode is clamp-to-edge.

## Parity expectations

- Desktop and web should produce visually comparable output for the same baked volume, camera state, `k`, exposure, step count, and LUT.
- A step-count change is a renderer quality/performance change and should be reviewed visually, especially when comparing desktop and web.
- Small differences from shader language or backend texture handling are acceptable only if they do not change the contract above in a user-visible way.
