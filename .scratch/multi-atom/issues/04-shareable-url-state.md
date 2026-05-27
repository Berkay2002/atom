# Shareable URL state

Status: ready-for-agent

## Parent

`.scratch/multi-atom/PRD.md`

## What to build

Make any `Scene` reproducible from a URL — sharing a link reproduces the exact view. Foundational for guided tours (issue 07) and a baseline expectation for any web demo.

Two pieces:

**`atom-core::scene` — shared encoder/decoder.** Add `fn encode(scene: &Scene) -> String` and `fn decode(s: &str) -> Result<Scene, DecodeError>`. The format is a versioned, compact, URL-safe string with a `v1:` prefix. Exact syntax is an implementation call (delimited form like `v1:H/1/2/1/0/eff/turbo/1.0/...` is fine for slice 1 — single atom, fixed field count). Per the spec, the format **must live in `atom-core`** so desktop and web produce and accept identical strings (otherwise shareable URLs silently break across clients).

Add two unit tests:

- **Round-trip:** for a representative sample of scenes (different elements, orbitals, view settings), `decode(encode(scene)) == scene`.
- **Fixed-string fidelity:** a hand-written string like `"v1:H/1/2/1/0/eff/turbo/1.0"` decodes to a Scene matching exactly-specified field values. This is the test that prevents format drift between desktop and web during future edits.

Decode errors should be friendly enough for a UI to display ("unknown element 'Xx'", "missing field at position N", "unsupported version 'v2'"), not just panic.

**Web routing glue — `web/src/lib/scene-url.ts`.** A thin module that:

- Reads the URL (search param like `?s=v1:...` or path segment, your call — pick what fits Next.js's app router conventions) on page load and hydrates the initial Scene.
- Subscribes to Scene changes (debounced) and pushes the new encoded string into the URL via `router.replace` so the address bar updates without adding history entries on every paint.
- Handles decode errors gracefully: malformed URLs fall back to the default Scene (single hydrogen at origin, `n=1, l=0, m=0`) and show a non-blocking toast/inline notice rather than crashing.

Desktop does not get URL routing in this slice — it's a web-only concern. But the encoder/decoder is shared, so a future "export this view as URL" button on desktop would just call `scene::encode` and copy to clipboard.

## Acceptance criteria

- [ ] `atom-core::scene::{encode, decode}` exist with `v1:` prefix
- [ ] Round-trip unit test passes for at least 4 representative scenes
- [ ] Fixed-string fidelity test passes — locks the format down
- [ ] Decode returns a typed `DecodeError`, not a panic, for malformed inputs
- [ ] Opening the app in the browser with a Scene URL hydrates the exact view
- [ ] Editing element / orbital / Bare-Z toggle on web updates the URL (debounced; no history spam)
- [ ] Malformed URLs fall back to the default Scene and show a user-visible notice
- [ ] WASM exports the codec functions so the web target uses the shared implementation

## Blocked by

`.scratch/multi-atom/issues/01-scene-shaped-bake-api.md`
