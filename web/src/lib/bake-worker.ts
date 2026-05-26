/// <reference lib="webworker" />
//
// Off-thread volume bake. The wasm-bindgen JS shim runs here so the
// hydrogen wavefunction math (rayon-free under wasm) never blocks the
// rendering thread.
//
// Protocol:
//   main -> worker:  { type: 'bake', n, l, m, res }
//   worker -> main:  { type: 'ready' }            (after init)
//                  | { type: 'bake-result', data: Float32Array,
//                      halfExtent: number, peak: number }
//
// The `data` Float32Array returned by atom-core's BakeResult is a *view*
// into wasm linear memory; sending it directly would alias memory that
// can move on the next allocation. We copy into a fresh ArrayBuffer and
// transfer ownership across the postMessage boundary.

import init, { bake } from '../../wasm/atom_core.js';

type BakeRequest = {
  type: 'bake';
  n: number;
  l: number;
  m: number;
  res: number;
};

type WorkerOut =
  | { type: 'ready' }
  | {
      type: 'bake-result';
      data: Float32Array;
      halfExtent: number;
      peak: number;
    };

const ctx = self as unknown as DedicatedWorkerGlobalScope;

let ready: Promise<void> | null = null;

function ensureReady(): Promise<void> {
  if (!ready) {
    ready = init().then(() => {
      const msg: WorkerOut = { type: 'ready' };
      ctx.postMessage(msg);
    });
  }
  return ready;
}

ctx.addEventListener('message', async (ev: MessageEvent<BakeRequest>) => {
  const req = ev.data;
  if (!req || req.type !== 'bake') return;
  await ensureReady();

  const result = bake(req.n, req.l, req.m, req.res);

  // Copy the wasm-memory view into an owned Float32Array before posting,
  // then free the Rust-side BakeResult.
  const view = result.data;
  const copy = new Float32Array(view.length);
  copy.set(view);
  const halfExtent = result.half_extent;
  const peak = result.peak;
  result.free();

  const out: WorkerOut = {
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
