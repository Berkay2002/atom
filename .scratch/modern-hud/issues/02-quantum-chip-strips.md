# Quantum chip strips replace n/l/m sliders

Status: ready-for-agent

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Implement the `chip_strip` custom widget in `ui_widgets` and use it for the three quantum-number axes (n, l, m). Replace the existing `egui::Slider` calls in `ui.rs::panel`.

Chip appearance (from PRD):

- rest: `--surface-mute` fill, `--text-primary` text
- selected: `--accent-dim` fill + 1 px `--accent` inset stroke
- disabled (invalid combo): rest style at 25 % alpha, click is a no-op
- hover: lighten fill by ~+0.04 alpha

Labels:

- n: integers `1..=6`
- l: spectroscopic `s p d f g h` (still bound 0..=n−1; chips for higher l are present but disabled when n is small)
- m: signed integers `−l..=+l`; chips for out-of-range m are disabled

Constraint logic stays identical to the slider version (`l > n−1 → clamp l`, `m → clamp(−l, l)`).

## Acceptance criteria

- [ ] `chip_strip` widget exists in `ui_widgets` and returns the user's selection via `Response` or a return value.
- [ ] Each disabled chip is visibly dimmed and ignores clicks.
- [ ] Changing n updates which l chips are enabled; changing l updates which m chips are enabled.
- [ ] Rebake still triggers on any (n, l, m) change.
- [ ] No `egui::Slider` calls remain for n, l, or m.
- [ ] The three chip rows render under a "Quantum numbers" uppercase label per the visual system.

## Blocked by

- `.scratch/modern-hud/issues/01-design-tokens-and-card-frame.md`
