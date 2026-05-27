# Guided tours v1

Status: ready-for-agent

## Parent

`.scratch/multi-atom/PRD.md`

## What to build

Add a guided-tours system to the web demo: a tour is an ordered list of `{scene, caption}` steps the user advances through with prev/next controls. Tours are JSON content authored by hand; this slice builds the engine plus 2 starter tours.

Tour JSON schema (lives in `web/public/tours/<slug>.json`):

```json
{
  "name": "Shielding across period 2",
  "description": "Watch the 2p orbital shrink as effective nuclear charge grows from boron to fluorine.",
  "steps": [
    { "scene": "v1:B/2/1/0/eff/turbo/1.0", "caption": "Boron's 2p — Z_eff ≈ 2.6. The outermost electron feels only ~2.6 of boron's 5 protons because the inner shell screens the rest." },
    { "scene": "v1:C/2/1/0/eff/turbo/1.0", "caption": "Carbon's 2p — Z_eff ≈ 3.25. One more proton, same shielding electrons, so the cloud pulls tighter." },
    { "scene": "v1:N/2/1/0/eff/turbo/1.0", "caption": "Nitrogen 2p — Z_eff ≈ 3.9..." },
    { "scene": "v1:O/2/1/0/eff/turbo/1.0", "caption": "..." },
    { "scene": "v1:F/2/1/0/eff/turbo/1.0", "caption": "..." }
  ]
}
```

UI on web:

- A "Tours" entry point — a labeled section in the controls panel listing available tours by name. Selecting a tour enters tour mode.
- In tour mode: a caption bar (somewhere visually prominent — bottom or top of the canvas) showing the current step's caption text, with Prev / Next / Exit controls. Step indicator like "Step 2 of 5".
- Each Prev/Next click decodes the step's `scene` string and applies it as the current Scene, triggering a re-bake. Step transitions are **hard cuts** in slice 1 (no animated morph — explicitly deferred per the spec).
- Exiting tour mode returns to free-sandbox mode with the last tour step's Scene preserved as the current state.
- The URL should reflect tour state: e.g., `?tour=shielding-period-2&step=3`. Opening such a URL deep-links into tour mode at that step.

Starter tours to ship in this slice:

1. **`shielding-period-2.json`** — "Shielding across period 2" (B → C → N → O → F at the same 2p orbital). Demonstrates Z_eff trends.
2. **`orbital-shapes.json`** — "Orbital shapes 1s through 3d" (a walking tour of distinct orbital geometries at the same element, e.g., Argon, picking interesting `(n, l, m)` combinations to show s/p/d shapes).

Captions for both tours are part of this slice — write them with care since they're the user-facing content.

Tour discovery: hard-code the list of available tour files in a small `web/public/tours/index.json` or equivalent. The spec defers a tour-authoring UI; for now adding a tour means dropping a JSON file and updating the index.

Tour content lives **only on web** per the spec ("Tours are web-demo content"); desktop does not get a tours UI in this slice.

## Acceptance criteria

- [ ] `web/public/tours/shielding-period-2.json` and `web/public/tours/orbital-shapes.json` exist with valid step content
- [ ] `web/public/tours/index.json` (or equivalent) lists available tours
- [ ] Tour selector visible in web controls panel
- [ ] Entering a tour displays the first step's caption and renders its Scene
- [ ] Prev / Next advance through steps; out-of-bounds disables the respective button
- [ ] Step indicator shows current position (e.g., "Step 2 of 5")
- [ ] Exiting tour mode preserves the last viewed Scene
- [ ] URL reflects tour state (`?tour=...&step=...`); deep-link into a specific step works
- [ ] Tour step transitions are hard cuts (no animation expected in this slice)
- [ ] Captions are pedagogically reasonable — proofread, not placeholder text

## Blocked by

`.scratch/multi-atom/issues/04-shareable-url-state.md`
