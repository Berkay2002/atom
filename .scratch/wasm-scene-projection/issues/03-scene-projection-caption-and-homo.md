# Scene projection caption and HOMO

Status: ready-for-agent

## Parent

PRD: `.scratch/wasm-scene-projection/PRD.md`

## What to build

Route caption and HOMO-related browser projection behavior through the same shared Scene and element presentation concepts as the codec and bake paths. Delete or explicitly justify any remaining TypeScript snapshot of shared element facts.

## Acceptance criteria

- [ ] Browser caption generation still matches shared Rust caption behavior.
- [ ] Browser HOMO snapping still matches shared element presentation facts.
- [ ] Any remaining TypeScript projection of HOMO or element facts is generated, documented, or covered by parity tests.
- [ ] Caption and HOMO paths do not introduce another unrelated tuple-shaped WASM interface.
- [ ] Existing web caption and element behavior remain unchanged.

## Blocked by

- `.scratch/wasm-scene-projection/issues/01-scene-projection-codec.md`
- `.scratch/element-presentation/issues/01-shared-element-presentation-facts.md`
