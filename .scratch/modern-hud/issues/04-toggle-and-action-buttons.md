# Toggle switch + action buttons for Footer group

Status: ready-for-agent

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Implement two new custom widgets and wire them into the card footer:

- `toggle_switch` — 24×14 px pill, slides white circle left→right on activation. Replaces `egui::checkbox` for auto-rotate.
- `action_button` — rounded rect with optional keyboard-shortcut hint. Replaces `egui::Button` for "Fit camera" and "Screenshot".

Visual states match the PRD's component specs (rest / hover / pressed).

The shortcut hint (`F`, `S`) renders to the right of the label in `--text-tertiary` at a smaller size.

## Acceptance criteria

- [ ] `toggle_switch` and `action_button` exist in `ui_widgets`.
- [ ] Auto-rotate now uses `toggle_switch`; functional behavior unchanged.
- [ ] Fit / Capture buttons render with the new look. Existing keyboard shortcuts (F, S) still work.
- [ ] No `egui::checkbox` or `egui::Button` calls remain in the new card footer.

## Blocked by

- `.scratch/modern-hud/issues/01-design-tokens-and-card-frame.md`
