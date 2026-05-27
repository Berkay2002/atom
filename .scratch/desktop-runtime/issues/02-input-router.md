# Desktop input router

Status: ready-for-agent

## Parent

PRD: `.scratch/desktop-runtime/PRD.md`

## What to build

Extract desktop mouse and keyboard input routing from frame rendering. The input seam should translate winit events into camera movement and UI commands while preserving egui focus handling and existing shortcuts.

## Acceptance criteria

- [ ] Mouse drag still orbits the camera as before.
- [ ] Mouse wheel still zooms the camera as before.
- [ ] `F`, `S`, `H`, and Space shortcuts still perform Fit, screenshot request, HUD toggle, and auto-rotate toggle when egui does not want keyboard input.
- [ ] Input ignored by egui remains consumed as before.
- [ ] If the input router exposes pure command mapping, tests cover shortcut-to-command behavior.
- [ ] Manual desktop verification confirms mouse drag, wheel, and shortcuts.

## Blocked by

- `.scratch/desktop-runtime/issues/01-scene-bake-state.md`
