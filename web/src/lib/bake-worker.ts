/// <reference lib="webworker" />
//
// Off-thread volume bake. The wasm-bindgen JS shim runs here so the
// hydrogen wavefunction math (rayon-free under wasm) never blocks the
// rendering thread.
//
// Protocol (single-shot per worker — cancellation = terminate + respawn,
// orchestrated by `BakeClient`):
//
//   main -> worker:  { type: 'requestBake',
//                      scene: { elementZ, orbital: { n, l, m },
//                               view: { useBareZ } }, res }
//   worker -> main:  { type: 'bake-result', data: Float32Array,
//                      halfExtent: number, peak: number }
//
// Issue 02 of the multi-atom direction carries the element atomic number
// and the bare-Z toggle across the boundary. The JS↔WASM boundary is
// still a thin parameter tuple (no JSON serde) — we mirror the Scene
// shape only in the message envelope so future multi-atom slices can
// grow it without another protocol rename.
//
// The `data` Float32Array returned by atom-core's BakeResult is a *view*
// into wasm linear memory; sending it directly would alias memory that
// can move on the next allocation. We copy into a fresh ArrayBuffer and
// transfer ownership across the postMessage boundary.

import init, { bake_scene } from '../../wasm/atom_core.js';

export type SceneRequest = {
  /** Atomic number of the selected element (1..=18). */
  elementZ: number;
  orbital: { n: number; l: number; m: number };
  /** View flags. `useBareZ` overrides Slater shielding with the bare atomic number. */
  view: { useBareZ: boolean };
};

export type BakeRequest = {
  type: 'requestBake';
  scene: SceneRequest;
  res: number;
};

export type BakeResult = {
  type: 'bake-result';
  data: Float32Array;
  halfExtent: number;
  peak: number;
};

const ctx = self as unknown as DedicatedWorkerGlobalScope;

let ready: Promise<void> | null = null;

function ensureReady(): Promise<void> {
  if (!ready) ready = init().then(() => undefined);
  return ready;
}

ctx.addEventListener('message', async (ev: MessageEvent<BakeRequest>) => {
  const req = ev.data;
  if (!req || req.type !== 'requestBake') return;
  await ensureReady();

  const { n, l, m } = req.scene.orbital;
  const { elementZ } = req.scene;
  const { useBareZ } = req.scene.view;
  const result = bake_scene(elementZ, n, l, m, useBareZ, req.res);

  // Copy the wasm-memory view into an owned Float32Array before posting,
  // then free the Rust-side BakeResult.
  const view = result.data;
  const copy = new Float32Array(view.length);
  copy.set(view);
  const halfExtent = result.half_extent;
  const peak = result.peak;
  result.free();

  const out: BakeResult = {
    type: 'bake-result',
    data: copy,
    halfExtent,
    peak,
  };
  ctx.postMessage(out, { transfer: [copy.buffer] });
});

// Kick off WASM init eagerly so the first bake request doesn't pay the
// instantiation cost serially with the bake.
void ensureReady();

export {};
