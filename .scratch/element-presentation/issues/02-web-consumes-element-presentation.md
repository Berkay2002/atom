# Web consumes element presentation

Status: ready-for-agent

## Parent

PRD: `.scratch/element-presentation/PRD.md`

## What to build

Update the web target to consume shared element presentation facts instead of maintaining parallel element symbol, HOMO, or periodic-position knowledge where practical. The web UI should still render its own controls, but it should not own domain facts that now live behind the shared presentation interface.

## Acceptance criteria

- [ ] The web element picker renders from shared presentation facts or a generated projection of them.
- [ ] The web element-change snap uses the shared HOMO source or a generated snapshot with parity coverage.
- [ ] Existing web behavior for element selection, captions, and periodic layout is preserved.
- [ ] Any remaining TypeScript snapshot is documented as a projection and covered by parity tests against shared facts.
- [ ] Web tests covering element HOMO or presentation snapshots pass.

## Blocked by

- `.scratch/element-presentation/issues/01-shared-element-presentation-facts.md`
