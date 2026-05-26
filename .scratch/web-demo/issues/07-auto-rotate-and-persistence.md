# Auto-rotate + localStorage persistence

Status: ready-for-agent

## What to build

Two small features that turn the demo from "interactive page" into "comes back to where you left it and looks alive on its own."

**Auto-rotate.** Add an "Auto-rotate" toggle to `Controls`. When on, `AtomCanvas` adds a small constant to `camera.azimuth` each frame (e.g. ~30°/sec at 60fps → +0.0087 rad/frame). User pointer drag still works while auto-rotate is on — the user's input is applied on top of the rotation. Auto-rotate state is part of the React state owned by the page.

**localStorage persistence.** On mount, the page rehydrates these values from `localStorage` (if present) and uses them as initial state instead of defaults:

- `(n, l, m)` — current orbital
- `colormap` — selected colormap name
- `autoRotate` — boolean

Writes happen via a debounced effect (~300ms) so rapid slider drags don't thrash localStorage. Invalid stored values (e.g. `l ≥ n`) are clamped or rejected on rehydration — the page must never load into a broken state.

A clean first-visit (no stored values) loads the defaults from slice 05: `(n=3, l=2, m=1)`, `INFERNO` colormap, auto-rotate off.

## Acceptance criteria

- [ ] "Auto-rotate" toggle exists in `Controls`; default is off
- [ ] When on, the camera azimuth advances each frame at a constant rate that completes a full revolution in ~12 seconds
- [ ] User pointer drag composes with auto-rotate (drag applies on top of the auto-spin; releasing returns to auto-rotation from the new heading)
- [ ] On mount, the page reads `(n, l, m)`, colormap, and autoRotate from localStorage and uses them as initial state if present and valid
- [ ] Invalid stored `(n, l, m)` combinations (e.g. `l ≥ n` or `|m| > l`) are clamped on rehydration
- [ ] Changes to any of the persisted values are written to localStorage with a ~300ms debounce
- [ ] First-visit defaults: `(n=3, l=2, m=1)`, `INFERNO`, autoRotate=false
- [ ] No regression in the debounced bake pipeline from slice 05

## Blocked by

- Issue 06 (presets + colormaps)
