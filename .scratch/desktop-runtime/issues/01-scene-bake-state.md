# Desktop Scene bake state

Status: ready-for-agent

## Parent

PRD: `.scratch/desktop-runtime/PRD.md`

## What to build

Extract desktop Scene bake state from the broad runtime so current Scene selection, current half-extent, peak, resolution, and rebake decisions have locality. The slice should preserve sync baking and all current user-facing behavior.

## Acceptance criteria

- [ ] Element, Orbital, Bare Z, and resolution changes still trigger rebake.
- [ ] Colormap, k, exposure, camera, HUD visibility, and shortcuts do not trigger unnecessary rebake.
- [ ] Current half-extent and peak remain updated from the baked Volume.
- [ ] Camera fit still uses the baked Volume half-extent.
- [ ] Pure rebake invalidation behavior is covered by tests if exposed as a testable interface.
- [ ] Manual desktop verification confirms element change, Orbital change, Bare Z, resolution, and Fit behavior.

## Blocked by

None - can start immediately
