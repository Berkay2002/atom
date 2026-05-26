# Bake-in-flight indicator (shimmer / progress overlay)

Status: ready-for-agent

## What to build

When a bake is in flight (between the visitor's input and the new volume arriving on the GPU), the canvas shows a subtle indicator that work is happening. Without this, fast scrubbing feels broken on slow devices — the old orbital lingers for up to ~2 seconds before snapping to the new one with no visual hint that the system saw the input.

The indicator is driven by the `baking: boolean` already exposed by `useDebouncedBake` from slice 05 — no new state machine. Pick the cheapest visual that still reads as "working":

- A thin animated progress bar across the top of the canvas (slow indeterminate sweep), or
- A subtle shimmer/grain overlay on top of the canvas, or
- A small badge in a corner ("Baking…").

The indicator appears within one frame of `baking` becoming true and disappears within one frame of it becoming false. It must not block pointer events — the visitor can still drag the camera while a bake is pending.

## Acceptance criteria

- [ ] An overlay element (progress bar, shimmer, or badge) is visible while `useDebouncedBake.baking === true`
- [ ] The overlay appears immediately when `baking` flips to true and disappears immediately when it flips to false
- [ ] The overlay does not capture pointer events (`pointer-events: none` or equivalent)
- [ ] During a long bake (e.g. on a throttled CPU profile), the visitor still has camera control via pointer drag
- [ ] The overlay is visually subtle — it should not be the loudest thing on the page; the orbital remains the focus
- [ ] No regression in any prior slice; the indicator is purely additive

## Blocked by

- Issue 05 (controls + debounced bake)
