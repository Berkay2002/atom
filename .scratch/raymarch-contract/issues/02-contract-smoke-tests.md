# Ray-march contract smoke tests

Status: ready-for-agent

## Parent

PRD: `.scratch/raymarch-contract/PRD.md`

## What to build

Add narrow, non-GPU tests or checks for deterministic parts of the ray-march contract where the codebase can do so cheaply. The goal is to catch formula and LUT drift without mocking wgpu or WebGL2 or building a screenshot-diff framework.

## Acceptance criteria

- [ ] CPU-side tests cover intensity mapping for representative sums, k values, and exposure values if the formula is factored into a testable helper.
- [ ] CPU-side tests cover LUT interpolation byte behavior if the interpolation is factored into a testable helper.
- [ ] Any shader smoke validation added by this slice is cheap and does not require mocking full graphics APIs.
- [ ] No wgpu or WebGL2 mocks are introduced.
- [ ] Manual visual parity remains documented as the verification path for full renderer behavior.

## Blocked by

- `.scratch/raymarch-contract/issues/01-document-raymarch-contract.md`
