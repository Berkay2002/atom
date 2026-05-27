# Shared element presentation facts

Status: ready-for-agent

## Parent

PRD: `.scratch/element-presentation/PRD.md`

## What to build

Add a shared element presentation interface that exposes UI-ready facts for H-Ar without embedding UI widget behavior. It should include atomic number, symbol, display name, electron configuration display string, HOMO Orbital, and periodic-table slot metadata.

## Acceptance criteria

- [ ] The shared interface exposes presentation facts for all H-Ar elements.
- [ ] HOMO remains `m = 0` and matches the current intended table.
- [ ] Periodic-table slots are valid and unique within the H-Ar layout.
- [ ] Unknown or unsupported elements have explicit documented fallback behavior.
- [ ] Tests cover contiguous H-Ar support, unique symbols, HOMO values, periodic-table slots, and representative fallback behavior.

## Blocked by

None - can start immediately
