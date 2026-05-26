# Controls + debounced bake (n/l/m sliders, auto-fit on bake)

Status: ready-for-agent

## What to build

Add the parameter UI and the debounce-and-cancel pipeline that keeps the bake responsive while the visitor scrubs sliders.

`components/Controls.tsx` renders n (1..6), l (0..n−1), m (−l..l) sliders. When n changes, l clamps to `min(l, n-1)`; when n or l changes, m clamps to `max(-l, min(m, l))`. Invalid `(n, l, m)` combinations are never emitted from `Controls` — the WASM layer is not responsible for validation.

`hooks/useDebouncedBake.ts` encapsulates the bake state machine. When params change: wait 150ms of quiet; if another change arrives in that window, reset the timer. When the timer fires, call `worker.cancel()` (terminate the in-flight worker if any) then `worker.requestBake(latestParams)` on a freshly-spawned worker. Returns `{ volume, baking }`. The worker dependency is injected so it can be substituted in tests.

`bake-worker.ts` from slice 03 gets two additions: a `cancel()` method that terminates the worker, and a `requestBake()` that returns a `Promise<Volume>`. Cancellation = terminate-and-respawn; there's no in-WASM checkpointing.

`AtomCanvas` consumes `useDebouncedBake({n, l, m, res: 96}, worker)`, hands the resulting `volume` to the renderer, and calls `camera.fit(halfExtent)` every time a new volume arrives so the orbital is always framed.

Write Vitest tests for `useDebouncedBake` with fake timers and React Testing Library:

1. A single param change fires exactly one bake after 150ms.
2. Two param changes 50ms apart fire exactly one bake, 150ms after the second change, with the second params.
3. A param change while a bake is in flight cancels the in-flight bake and starts a new one with the latest params.
4. `baking` is `true` from request start to result delivery.

Default starting orbital: `(n=3, l=2, m=1)` to match the desktop app's `UiState::default()`.

## Acceptance criteria

- [ ] `components/Controls.tsx` renders n/l/m sliders with the bounds and clamping rules above
- [ ] `hooks/useDebouncedBake.ts` implements the 150ms debounce + cancel-in-flight state machine described above
- [ ] `bake-worker.ts` exposes `requestBake()` returning a Promise and `cancel()` that terminates the worker
- [ ] `AtomCanvas` calls `camera.fit(halfExtent)` on every new volume so framing always works at any (n, l, m)
- [ ] Default page load shows `(n=3, l=2, m=1)`
- [ ] Scrubbing n from 1 to 6 quickly produces exactly one bake (the final value), not six
- [ ] All four Vitest tests for `useDebouncedBake` pass
- [ ] No visual regression — orbital still renders correctly with the orbit camera from slice 04

## Blocked by

- Issue 03 (tracer bullet)
