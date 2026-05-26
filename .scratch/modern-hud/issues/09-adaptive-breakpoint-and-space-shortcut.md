# Adaptive breakpoint + Space shortcut

Status: ready-for-agent

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Two small wiring changes that make the HUD behave well across aspect ratios and add the auto-rotate shortcut promised in the PRD's keyboard table.

**Adaptive breakpoint:** when the viewport width is below 600 px, force `card_expanded = false` regardless of the user's chevron state. When the user resizes back to ≥ 600 px, restore the previous `card_expanded` value (i.e. don't lose their explicit collapse if they collapsed manually before narrowing).

Implementation note: keep a separate `user_card_expanded: bool` mirroring user intent, and derive the rendered state as `user_card_expanded && viewport_w >= 600 px`. No animation — hard cutoff is acceptable per PRD risk note ("smooth this only if it actually feels bad in practice").

**Space shortcut:** pressing `Space` (when egui has no keyboard focus) toggles `UiState::auto_rotate`. Same focus-guard pattern as the `H` shortcut from issue 05.

## Acceptance criteria

- [ ] Resizing the window below 600 px width auto-collapses the card. Resizing back restores the user's prior expanded state.
- [ ] The chevron remains clickable in collapsed mode; if the user expands while < 600 px the card opens (manual override wins, breakpoint only forces collapse from default).
- [ ] `Space` toggles auto-rotate. Does not fire while text input has focus.

## Blocked by

- `.scratch/modern-hud/issues/04-toggle-and-action-buttons.md`
- `.scratch/modern-hud/issues/06-card-collapse-state.md`
