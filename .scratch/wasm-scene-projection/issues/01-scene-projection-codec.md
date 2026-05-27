# Scene projection codec

Status: ready-for-agent

## Parent

PRD: `.scratch/wasm-scene-projection/PRD.md`

## What to build

Introduce a browser Scene projection for the WASM encode/decode path while preserving the existing Scene URL byte format. The TypeScript URL adapter should cross the WASM seam through the projection instead of repeating a bespoke flat tuple shape for codec operations.

## Acceptance criteria

- [ ] Encoding through the projection produces the same Scene URL strings as before.
- [ ] Decoding existing Scene URL strings through the projection produces the same browser-facing state as before.
- [ ] Decode errors remain readable and compatible with current UI handling.
- [ ] The fixed Scene URL fidelity case remains locked byte-for-byte.
- [ ] Tests cover projection encode/decode round trips and fixed-string fidelity through the WASM seam.

## Blocked by

None - can start immediately
