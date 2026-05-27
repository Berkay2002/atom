# Desktop runtime

Status: ready-for-agent

## Problem Statement

The desktop app's frame orchestration is concentrated in one broad mutable runtime struct. It initializes wgpu and egui, owns camera state, handles input, runs UI, decides rebakes, uploads volumes and LUTs, updates uniforms, captures screenshots, computes FPS, and manages frame presentation. The implementation works, but its interface is shallow for maintainers: changing frame behavior requires understanding many ordering constraints at once.

## Solution

Split the desktop frame orchestration into deeper modules around runtime responsibilities while preserving behavior. The ApplicationHandler should remain thin. A desktop runtime module owns frame progression and delegates to smaller seams for scene bake state, input routing, GPU rendering, HUD interaction, and screenshot capture. This is architecture work, not a visual redesign.

## User Stories

1. As a desktop user launching the app, I want the same default Scene to appear, so that this refactor does not change startup behavior.
2. As a desktop user dragging the mouse, I want orbit behavior unchanged, so that camera control stays familiar.
3. As a desktop user using the mouse wheel, I want zoom behavior unchanged, so that inspection remains predictable.
4. As a desktop user pressing Fit, I want the camera to fit the current volume half-extent, so that high-Z and low-Z atoms frame correctly.
5. As a desktop user changing element or Orbital controls, I want the volume to rebake exactly when required, so that visual updates stay correct.
6. As a desktop user changing colormap, k, exposure, or camera, I want no unnecessary rebake, so that interaction stays responsive.
7. As a desktop user taking a screenshot, I want the PNG to capture the orbital without HUD overlay, so that the existing screenshot contract stays intact.
8. As a desktop user toggling HUD visibility, I want shortcuts and eye toggle behavior unchanged, so that existing workflow remains intact.
9. As a developer modifying screenshot behavior, I want screenshot readback in one module, so that GPU buffer rules are local.
10. As a developer modifying input behavior, I want input routing separate from render submission, so that camera and shortcut changes are easier to review.
11. As a developer modifying rebake behavior, I want Scene bake invalidation in one place, so that UI changes cannot accidentally trigger wrong bakes.
12. As a developer modifying renderer calls, I want uniform updates and resource replacement ordered by a clear runtime interface, so that frame bugs are easier to locate.
13. As a future agent adding async 512-cubed bake, I want a clear bake state seam, so that double-buffering can be introduced without rewriting ApplicationHandler.

## Implementation Decisions

- Preserve `cargo run --release` behavior from the repo root.
- Keep visual verification as the main verification path for renderer, camera, and UI.
- Keep the desktop renderer adapter in wgpu; do not introduce a cross-target renderer abstraction.
- Extract frame orchestration so ApplicationHandler delegates to a runtime interface.
- Extract scene bake state so current element, Orbital, half-extent, peak, and rebake decisions have locality.
- Extract input routing so mouse and keyboard handling is isolated from GPU frame submission.
- Extract screenshot capture/readback so PNG naming, buffer alignment, and overlay ordering have locality.
- Keep egui painting and custom widgets in existing UI modules unless a small move is required for runtime seams.
- Keep current sync bake behavior; async bake is out of scope.
- Do not change HUD visuals in this PRD; modern HUD is already a separate feature.

## Testing Decisions

- Good tests for this PRD should target pure orchestration decisions where possible, not wgpu or egui internals.
- If a bake-state module exposes pure invalidation logic, test which user changes require a rebake.
- If input routing exposes pure command output, test shortcut-to-command mapping separately from winit events.
- Preserve existing atom-core tests; do not invent renderer/UI unit tests that mock wgpu or egui.
- Verify desktop behavior visually with `cargo run --release`, not debug mode, after implementation.
- Manually verify screenshot output excludes HUD overlay.
- Manually verify Fit, S, H, Space, mouse drag, mouse wheel, element change, Bare Z, and colormap change.

## Out of Scope

- Async volume baking.
- Enabling 512-cubed default.
- Changing renderer algorithm.
- Changing HUD visual design.
- Changing screenshot file format.
- Sharing desktop camera with web camera through core.
- Adding new desktop features.

## Further Notes

The goal is locality around runtime behavior. Avoid extracting pass-through modules that merely rename existing calls; each new module should hide ordering or state complexity.
