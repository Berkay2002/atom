# Glass sliders + grid-resolution chips for Render group

Status: ready-for-agent

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Implement the `glass_slider` widget and apply it to the existing `k` (saturation) and `exposure` sliders in the card. Visual spec from the PRD:

- 3 px track, `--surface-mute` background
- fill from 0 → current value in `--accent-dim`
- 8 px circular knob in `--accent` with a 6 px outer glow (`--accent-dim`)
- right-aligned tabular-numeric value, `--text-primary`

Drag scrubs; clicking the track jumps the knob.

Also: replace the resolution `egui::ComboBox` with a chip strip using the existing `chip_strip` widget (labels `128³`, `256³`). 512³ stays deferred per PRD non-goals.

Group all three under a "Render" uppercase label.

## Acceptance criteria

- [ ] `glass_slider` widget exists in `ui_widgets` and matches the PRD spec.
- [ ] `k` and `exposure` use `glass_slider`. Functional behavior unchanged (same value ranges, same downstream wiring).
- [ ] Resolution chooser uses `chip_strip` with `128³` / `256³`. Rebake fires on change.
- [ ] No `egui::Slider` or `egui::ComboBox` calls remain in the Render group.

## Blocked by

- `.scratch/modern-hud/issues/01-design-tokens-and-card-frame.md`
- `.scratch/modern-hud/issues/02-quantum-chip-strips.md`
