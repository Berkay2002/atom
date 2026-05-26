# Card collapse state + chevron button

Status: ready-for-agent

## Parent

`.scratch/modern-hud/PRD.md`

## What to build

Add a collapse chevron to the card's title row (right-aligned, next to the orbital name). When the user clicks it, the card collapses to its title-only state: the chevron flips from `−` to `+`, and all groups (Quantum numbers, Colormap, Render, Footer) are hidden.

State lives on `UiState::card_expanded: bool`, defaults to `true`, in-memory only.

The collapsed card keeps the same background, padding, and title as the expanded card — it's just shorter. The title block still has no separator or background tint.

## Acceptance criteria

- [ ] `UiState::card_expanded` field added; defaults to `true`.
- [ ] Title row renders a chevron button (`−` when expanded, `+` when collapsed).
- [ ] Clicking the chevron toggles `card_expanded` and the visible body responds immediately.
- [ ] Collapsed card visually matches the v5 mockup's collapsed state.
- [ ] When `hud_visible` is false (from issue 05), the card doesn't render at all regardless of `card_expanded`.

## Blocked by

- `.scratch/modern-hud/issues/01-design-tokens-and-card-frame.md`
