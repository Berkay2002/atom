# Scale readout restyle + preset chip strip

Status: ready-for-agent

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Two adjacent bottom-row HUD elements:

**Scale readout (bottom-left):** restyle the existing scale-bar logic from `ui.rs::hud` (`ui.rs:138-196`). Keep the math identical. Update typography to match the PRD:

- 120 px white bar (narrower than current 200 px), 1.5 px stroke, white end caps
- inline text `{a₀:.1} a₀ · {nm:.3} nm` followed by a small-cap `BOX ±{half:.1} a₀` label in `--text-tertiary`
- add a text-shadow so the readout stays legible over bright orbital lobes (no card behind it)

**Preset chip strip (bottom-right):** replace the existing preset `egui::ComboBox` (`ui.rs:55-63`) with a horizontal strip of pill-shaped chips floating in the bottom-right corner (16 px inset). One chip per entry in `PRESETS`. The active chip is highlighted iff its `(n, l, m)` exactly matches the current state (so scrubbing quantum chips can incidentally highlight a preset). Clicking a chip sets `n, l, m`.

If the strip would overlap the scale readout, the overflow tail collapses behind a `More…` chip. Clicking `More…` opens a popover (`egui::popup` family — exact API to verify during implementation) showing the remaining presets.

## Acceptance criteria

- [ ] Scale readout renders with the new typography and 120 px bar.
- [ ] Scale text uses tabular numerics and includes the `BOX ±N a₀` label.
- [ ] Text-shadow is visible enough to remain legible over a max-brightness inferno lobe.
- [ ] Preset chips render at the bottom-right; one per entry in `PRESETS`.
- [ ] The chip matching the current `(n, l, m)` is highlighted; otherwise none is highlighted.
- [ ] Clicking a chip applies its `(n, l, m)` and triggers a rebake.
- [ ] When the strip would overlap the scale readout, a `More…` chip appears and the tail moves into a popover.
- [ ] No `egui::ComboBox` call remains for presets.

## Blocked by

- `.scratch/modern-hud/issues/01-design-tokens-and-card-frame.md`
- `.scratch/modern-hud/issues/02-quantum-chip-strips.md`
