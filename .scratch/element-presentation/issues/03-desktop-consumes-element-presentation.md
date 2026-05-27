# Desktop consumes element presentation

Status: ready-for-agent

## Parent

PRD: `.scratch/element-presentation/PRD.md`

## What to build

Update the desktop target to consume shared element presentation facts for element labels and selection metadata. The desktop UI should keep its egui-specific painting and layout, but it should no longer carry an independent element label table when shared presentation facts can provide it.

## Acceptance criteria

- [ ] Desktop element labels are sourced from the shared presentation facts.
- [ ] Desktop element selection still snaps to HOMO exactly as before.
- [ ] Desktop captions remain generated from shared core language.
- [ ] Desktop UI visual behavior remains unchanged except for eliminating duplicate facts.
- [ ] Manual verification with `cargo run --release` confirms element picker, caption, Bare Z, and rebake behavior still work.

## Blocked by

- `.scratch/element-presentation/issues/01-shared-element-presentation-facts.md`
