# Screenshot and frame runtime

Status: ready-for-agent

## Parent

PRD: `.scratch/desktop-runtime/PRD.md`

## What to build

Finish the desktop runtime split by moving screenshot readback and frame orchestration into deeper seams. ApplicationHandler should delegate to a runtime interface, while screenshot capture keeps its existing orbital-only contract and frame submission keeps the current ordering.

## Acceptance criteria

- [ ] ApplicationHandler delegates frame progression to a desktop runtime interface.
- [ ] Screenshot capture still copies before egui overlay is rendered.
- [ ] Screenshot filenames still include element and Orbital identifiers.
- [ ] Renderer uniform update, ray-march draw, egui draw, texture cleanup, queue submit, and frame present happen in the same effective order as before.
- [ ] Existing atom-core tests still pass.
- [ ] Manual desktop verification with `cargo run --release` confirms rendering, HUD, screenshot, and window resize behavior.

## Blocked by

- `.scratch/desktop-runtime/issues/01-scene-bake-state.md`
- `.scratch/desktop-runtime/issues/02-input-router.md`
