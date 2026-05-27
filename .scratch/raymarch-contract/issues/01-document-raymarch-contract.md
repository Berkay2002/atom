# Document ray-march contract

Status: ready-for-agent

## Parent

PRD: `.scratch/raymarch-contract/PRD.md`

## What to build

Create a concise ray-march contract that names the shared semantics desktop WGSL and web GLSL must preserve: volume data layout, coordinate space, camera inputs, ray-box intersection, step count, accumulation, intensity mapping, and LUT sampling. The contract must explicitly state that renderer adapters remain separate under ADR-0002.

## Acceptance criteria

- [ ] The contract documents normalized volume data, row-major x-fastest layout, cubic half-extent, and texture coordinate mapping.
- [ ] The contract documents camera world position and inverse view-projection semantics.
- [ ] The contract documents slab intersection against the volume cube.
- [ ] The contract documents march count and step-length behavior.
- [ ] The contract documents `density * step_length` accumulation and `exposure * (1 - exp(-k * sum))` intensity mapping.
- [ ] The contract documents LUT interpolation expectations.
- [ ] The contract references ADR-0002 and states that this is not a shared renderer abstraction.

## Blocked by

None - can start immediately
