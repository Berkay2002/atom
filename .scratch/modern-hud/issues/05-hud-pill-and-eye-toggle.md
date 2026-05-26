# HUD pill + eye toggle + hide-all shortcut

Status: ready-for-agent

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Replace the two existing `egui::Area`s that render FPS (top-right) and peak |ψ|² (bottom-right) with a single `hud_pill` widget rendered at top-right. The pill shows:

```
● 60.2 FPS │ peak |ψ|² 1.2e−3
```

`●` is a 6 px accent dot with a subtle glow. Background is `rgba(14, 12, 18, 0.6)`, 999 px corner radius.

Add an `eye_toggle` widget at the top-right corner (16 px inset, 28×28 px rounded square). The pill sits inset to its left to leave room. Clicking the eye toggles `UiState::hud_visible`. Pressing `H` (when egui has no keyboard focus) does the same.

When `hud_visible` is false: the card, pill, scale readout, and preset strip are all suppressed. The eye toggle remains visible but dimmed, with the open-eye glyph swapped for an empty-eye glyph.

`UiState::hud_visible: bool` defaults to `true` and is in-memory only (no persistence).

## Acceptance criteria

- [ ] `hud_pill` and `eye_toggle` exist in `ui_widgets`.
- [ ] FPS and peak |ψ|² render in a single top-right pill; the old bottom-right `egui::Area` for peak is gone.
- [ ] `UiState::hud_visible` field added; defaults to `true`.
- [ ] `H` keypress toggles `hud_visible`. Clicking the eye does the same.
- [ ] When `hud_visible` is false, the card / scale / preset strip don't render; only the dimmed eye toggle remains.
- [ ] `H` does not fire while a text field has focus (defensive — no text fields exist today but the guard belongs here).

## Blocked by

- `.scratch/modern-hud/issues/01-design-tokens-and-card-frame.md`
