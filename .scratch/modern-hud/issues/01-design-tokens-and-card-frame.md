# Design tokens + dark-glass card frame

Status: ready-for-human

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Introduce the visual foundation for the new HUD: a `ui_tokens` module holding all color / metric constants from the PRD's visual-system table, and a `card_frame` widget in a new `ui_widgets` module that paints the dark-glass card background (rounded rect, `--card-bg` fill, 1 px `--border` stroke).

Wrap the existing left-side controls in `card_frame` instead of `egui::SidePanel`. The existing built-in widgets (sliders, combo boxes, buttons) stay inside the new frame for now — only the container changes.

This slice is HITL because the mockups were CSS approximations. After this lands, the user eyeballs translucency, color, corner radius, and contrast against the orbital before AFK agents continue.

## Acceptance criteria

- [ ] New module `src/ui_tokens.rs` exports `Color32` / `f32` constants for every entry in the PRD's visual-system table.
- [ ] New module `src/ui_widgets.rs` exports `card_frame(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui))` or similar that renders the dark-glass background.
- [ ] `src/ui.rs::panel` uses `egui::Area` (top-left anchored, 16 px inset) + `card_frame` instead of `egui::SidePanel`. Existing controls render inside.
- [ ] Card width matches the PRD: `min(320 px, 38vw)`.
- [ ] Card has no visible header band or separator — the title region shares the card background uniformly (cf. the v5 mockup correction).
- [ ] App still launches, all controls still function, rebakes still trigger on (n, l, m, resolution) change.
- [ ] User has visually confirmed the look against the v5 mockup before this issue is closed.

## Blocked by

None - can start immediately
