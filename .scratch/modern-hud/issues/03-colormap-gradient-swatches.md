# Colormap gradient swatches replace combobox

Status: ready-for-agent

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Implement the `swatch_row` custom widget in `ui_widgets` and use it to pick the colormap. Replace the existing `egui::ComboBox` for colormap selection in `ui.rs::panel`.

Each swatch is a thin horizontal strip painted with the actual colormap LUT, sampled from `colormaps::ALL`. The selected swatch shows a 1 px white inset plus a 1 px `--accent` outer ring. Hover state is intentionally none — the gradient itself is the affordance.

Clicking a swatch sets `UiState::colormap_index`. No rebake (colormap changes only update the GPU LUT, not the volume bake).

## Acceptance criteria

- [ ] `swatch_row` widget exists in `ui_widgets`.
- [ ] One swatch per entry in `colormaps::ALL`. The widget reads the LUT, doesn't hard-code colors.
- [ ] Selected swatch is unambiguously marked (white inset + accent ring).
- [ ] No `egui::ComboBox` call remains for the colormap.
- [ ] Renders under a "Colormap" uppercase label per the visual system.

## Blocked by

- `.scratch/modern-hud/issues/01-design-tokens-and-card-frame.md`
