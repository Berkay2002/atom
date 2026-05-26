# Presets + colormaps (LUT texture, preset chips, picker)

Status: ready-for-agent

## What to build

Replace the hardcoded grayscale gradient from the tracer bullet with the same six colormaps the desktop app ships, and add a preset strip that jumps to recognizable orbitals.

`colormaps.ts` is a TypeScript module with six exported stop arrays: `INFERNO`, `VIRIDIS`, `MAGMA`, `PLASMA`, `GRAYSCALE`, `ELECTRON_BLUE`. RGB values are copied byte-for-byte from `crates/atom-desktop/src/colormaps.rs` so the web demo and the desktop app produce visually identical palettes.

The renderer gains a 1D LUT texture and a `setColormap(stops)` method that uploads a 256-entry RGBA8 texture interpolated from the 8-stop input (same linear-interpolation logic as `upload_lut_texture` in `render.rs`). The fragment shader replaces the inlined grayscale lookup with a `texture()` sample of the LUT. `setParams(k, exposure, steps)` keeps the existing defaults (k=5.0, exposure=1.0, steps=96).

`components/Controls.tsx` gains:

- A colormap picker (dropdown or swatch row — pick whichever is cheaper to build) listing the six maps by name. Defaults to `INFERNO`.
- A preset chip strip mirroring the desktop `PRESETS` constant verbatim: `1s`, `2s`, `2p_x`, `2p_y`, `2p_z`, `3d_xy`, `3d_xz`, `3d_yz`, `3d_(x²-y²)`, `3d_(z²)`, `4f_(z³)`. Clicking a chip sets `(n, l, m)` to the preset's values, triggering the normal debounced bake path from slice 05.

Preset clicks and slider drags share the same debounce window — a fast double-click on two different presets results in one bake of the second.

## Acceptance criteria

- [ ] `colormaps.ts` exports all six colormaps with byte-identical RGB values to `colormaps.rs`
- [ ] `renderer/raymarch.ts` uploads a 256-entry 1D RGBA8 LUT and samples it in the fragment shader
- [ ] `setColormap(stops)` re-uploads the LUT without recreating the WebGL program
- [ ] Switching colormaps recolors the existing volume without re-baking
- [ ] Colormap picker in `Controls` defaults to `INFERNO`; changing it updates the render
- [ ] Preset chip strip lists all 11 presets in the order from `PRESETS`
- [ ] Clicking a preset chip updates the n/l/m sliders and triggers a bake via the same debounced pipeline from slice 05
- [ ] No visual regression — every preset still renders correctly with the orbit camera and auto-fit from slices 04 and 05

## Blocked by

- Issue 05 (controls + debounced bake)
