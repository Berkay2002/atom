# Remove old SidePanel body and unused egui widget paths

Status: ready-for-agent

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Final scrub: by the time the prior nine slices have landed, every interactive element in `src/ui.rs::panel` is rendered through custom widgets in `ui_widgets`. Delete the dead code paths that used `egui::SidePanel`, `egui::Slider`, `egui::ComboBox`, `egui::Button`, and `egui::checkbox`.

Also remove any temporary scaffolding introduced in issue 01 that allowed old + new HUD to coexist (e.g. fall-back rendering paths, feature-flagged branches).

This is a pure-deletion slice. No new behavior. Tests, screenshots, and rebake triggers should remain identical to the post-issue-09 state.

## Acceptance criteria

- [ ] No call sites of `egui::SidePanel`, `egui::Slider`, `egui::ComboBox`, `egui::Button`, or `egui::checkbox` remain in `src/ui.rs`.
- [ ] `src/ui.rs` compiles without `dead_code` or `unused_imports` warnings.
- [ ] The app's behavior post-cleanup is observably identical to the post-issue-09 state.
- [ ] Any temporary scaffolding from issue 01 (coexistence shims, feature flags) is deleted.

## Blocked by

- `.scratch/modern-hud/issues/02-quantum-chip-strips.md`
- `.scratch/modern-hud/issues/03-colormap-gradient-swatches.md`
- `.scratch/modern-hud/issues/04-toggle-and-action-buttons.md`
- `.scratch/modern-hud/issues/05-hud-pill-and-eye-toggle.md`
- `.scratch/modern-hud/issues/06-card-collapse-state.md`
- `.scratch/modern-hud/issues/07-render-group-glass-slider-and-grid-chips.md`
- `.scratch/modern-hud/issues/08-scale-readout-and-preset-strip.md`
- `.scratch/modern-hud/issues/09-adaptive-breakpoint-and-space-shortcut.md`
