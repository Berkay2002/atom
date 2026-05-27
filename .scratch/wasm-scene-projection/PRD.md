# WASM scene projection

Status: ready-for-agent

## Problem Statement

The browser crosses the Rust/WASM seam through several flat tuple interfaces: bake, Scene URL encode/decode, caption, and HOMO behavior each project single-Atom Scene facts in their own shape. That keeps the seam shallow. Every new Scene field risks repeating the same parameter list across multiple exports and TypeScript adapters.

## Solution

Create a deeper browser scene projection interface that lets the web target cross the WASM seam through one Scene-shaped projection for the single-Atom slice. The projection should preserve the current behavior while reducing repeated tuple surfaces. This keeps ADR-0001 intact: Rust remains the source of truth for physics, Scene URL encoding, captions, and element facts that must not drift.

## User Stories

1. As a visitor baking an orbital in the browser, I want the same output as before, so that the refactor is invisible.
2. As a visitor sharing a Scene URL, I want encoded links to stay byte-compatible, so that existing links keep working.
3. As a visitor opening a shared Scene URL, I want decode errors to stay readable, so that malformed links remain debuggable.
4. As a visitor reading the caption, I want the caption to remain generated from the shared Rust logic, so that web and desktop language stays aligned.
5. As a visitor picking an element, I want HOMO snapping to continue matching Rust element data, so that the canvas lands on a valid iconic Orbital.
6. As a developer adding a future Scene field, I want to update one projection seam, so that the change does not fan out across every WASM export.
7. As a developer maintaining the web worker, I want the bake request shape to mirror Scene language, so that the worker protocol is not a parallel domain model.
8. As a developer maintaining URL state, I want the codec adapter to consume the same projection concept as bake and caption, so that Scene shape drift is obvious.
9. As a developer maintaining TypeScript tests, I want fewer parity tables, so that tests guard real cross-target contracts rather than duplicated constants.
10. As a developer reviewing ADR-0001 pressure, I want a clear rule for what belongs in the projection, so that atom-core does not become a general web helper crate.
11. As a developer changing element data, I want browser-facing element facts to come from Rust where drift is meaningful, so that web code does not silently fork the domain.
12. As a developer debugging WASM memory lifetime, I want volume data ownership and copying rules documented at the seam, so that callers do not hold invalid views.

## Implementation Decisions

- Keep the existing Scene URL format stable.
- Keep the existing baked volume data format stable: normalized `Float32Array`, resolution, half-extent, and peak.
- Introduce a browser projection concept that represents the slice-1 single-Atom Scene fields in one place.
- Use that projection as the input shape for bake and Scene URL encode paths.
- Use that projection as the input shape for caption paths where applicable.
- Keep decode output as the same projection shape, with a JS-friendly object for TypeScript callers.
- Decide whether HOMO should become a WASM export or remain a TypeScript snapshot with parity tests. Recommended default: expose enough Rust element presentation data to delete the duplicate HOMO table if it does not slow synchronous UI interactions.
- Keep renderer, camera, colormap LUTs, and UI chrome per-target as required by ADR-0001 and ADR-0002.
- Keep the projection intentionally narrow: no generic serialization framework and no arbitrary multi-Atom shape until multi-Atom UI actually lands.

## Testing Decisions

- Good tests cross the projection seam and assert stable externally visible behavior.
- Preserve or expand WASM parity tests for bake output against direct core bake output.
- Add projection tests for encode/decode round trips through the WASM seam.
- Add a fixed Scene URL fidelity test through the browser projection so byte format remains locked.
- Add caption parity coverage if caption still crosses the WASM seam independently.
- Add or preserve HOMO parity coverage depending on whether the TypeScript table remains.
- Do not mock WebGL or React for this PRD; this is about the Rust/WASM/TypeScript seam.

## Out of Scope

- Changing the physics model.
- Adding multi-Atom browser APIs.
- Changing Tour JSON.
- Sharing camera math through WASM.
- Sharing colormap LUTs through WASM unless required by a separate decision.
- Introducing a general JSON serde layer across WASM.

## Further Notes

This PRD is mainly about interface depth. It should reduce repeated parameter tuples without turning atom-core into a broad browser runtime.
