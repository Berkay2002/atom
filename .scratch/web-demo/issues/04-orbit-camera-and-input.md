# Orbit camera + pointer / touch input

Status: ready-for-agent

## What to build

Replace the hardcoded camera from the tracer bullet with a real orbit camera the visitor can drive. Mouse drag orbits, scroll wheel zooms, touch drag orbits, two-finger pinch zooms. Behavior matches `crates/atom-desktop/src/camera.rs` exactly — same state, same methods, same clamp limits, same sensitivity constants.

Two TypeScript modules:

- `camera/orbit-camera.ts` — Pure math, no DOM. State: `radius`, `azimuth`, `elevation`, `fovY`, `aspect`. Methods: `orbit(dxPx, dyPx)`, `zoom(factor)`, `fit(halfExtent)`, `position()`, `viewProj()`. Clamp `elevation` to ±(π/2 − 0.01); clamp `radius` to ≥ 0.1. Sensitivity constant matches the Rust value (0.005). Use `gl-matrix` for `lookAt` and `perspective`.
- `camera/pointer-input.ts` — Attaches Pointer Events (`pointerdown`/`pointermove`/`pointerup`) and `wheel` to a canvas. Translates events into `OrbitCamera.orbit` / `zoom` calls. Handles two-pointer pinch by tracking the inter-pointer distance and feeding the ratio to `zoom`. No DOM coupling in the camera module — input is the only thing that touches the canvas.

Wire both into `AtomCanvas`. The hardcoded camera transform is removed. The renderer receives `viewProj` and `position` from `OrbitCamera` each frame.

Write Vitest tests for `orbit-camera`, porting the three existing Rust tests in `camera.rs`: `fit` sets radius to twice the half-extent; elevation clamps at the poles under extreme drag; `zoom` multiplies radius.

## Acceptance criteria

- [ ] `camera/orbit-camera.ts` exists with the state and methods listed above; no DOM imports
- [ ] `camera/pointer-input.ts` attaches Pointer Events + wheel to a canvas and drives the camera
- [ ] Mouse drag on the canvas orbits smoothly with the same sensitivity as the desktop app
- [ ] Scroll wheel zooms in/out; `radius` floor of 0.1 prevents zooming through the origin
- [ ] Touch drag orbits on a phone or tablet; two-finger pinch zooms
- [ ] Elevation clamps just shy of ±π/2 — no gimbal flip at the poles
- [ ] `AtomCanvas` no longer uses any hardcoded camera state
- [ ] Vitest tests cover `fit`, elevation clamp, and `zoom`; all pass
- [ ] No regression in the tracer-bullet render — the 1s orbital still appears correctly with the new camera at its initial pose

## Blocked by

- Issue 03 (tracer bullet)
