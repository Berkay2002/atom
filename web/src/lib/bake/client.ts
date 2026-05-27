// Thin lifecycle wrapper around `bake-worker.ts`.
//
// Deep module: callers see only `requestBake / cancel / dispose`. The
// gnarly bits — worker spawn-on-demand, in-flight promise tracking,
// terminate-and-respawn for cancellation — live inside.
//
// Cancellation model:
//   * A new requestBake() implicitly supersedes any in-flight call. The
//     superseded promise rejects with `BakeCancelledError` and the
//     underlying Worker is terminated. The next requestBake spawns a
//     fresh worker.
//   * Explicit cancel() / dispose() do the same: reject in-flight,
//     terminate, clear state.
//
// The useDebouncedBake hook is expected to swallow `BakeCancelledError`
// — the cancellation only fires when a newer bake is already pending,
// so there is nothing useful to do with it.

import type { BakeRequest, BakeResult } from '../bake-worker';

export type BakeParams = {
  n: number;
  l: number;
  m: number;
  res: number;
};

export type Volume = {
  data: Float32Array;
  res: number;
  halfExtent: number;
  peak: number;
};

export class BakeCancelledError extends Error {
  constructor() {
    super('bake cancelled');
    this.name = 'BakeCancelledError';
  }
}

/** Factory so tests can substitute a fake Worker without touching DOM. */
export type WorkerFactory = () => Worker;

export interface IBakeClient {
  requestBake(params: BakeParams): Promise<Volume>;
  cancel(): void;
  dispose(): void;
}

export class BakeClient implements IBakeClient {
  private worker: Worker | null = null;
  private pending: {
    resolve: (v: Volume) => void;
    reject: (e: unknown) => void;
    res: number;
  } | null = null;

  constructor(private readonly factory: WorkerFactory) {}

  requestBake(params: BakeParams): Promise<Volume> {
    // Any in-flight bake is superseded.
    this.cancel();

    const worker = this.factory();
    this.worker = worker;

    return new Promise<Volume>((resolve, reject) => {
      this.pending = { resolve, reject, res: params.res };

      worker.addEventListener('message', (ev: MessageEvent<BakeResult>) => {
        const msg = ev.data;
        if (!msg || msg.type !== 'bake-result') return;
        // Drain: this worker only ever delivers one result, so tear it
        // down and clear pending before resolving.
        const p = this.pending;
        this.pending = null;
        if (this.worker === worker) {
          worker.terminate();
          this.worker = null;
        }
        if (p) {
          p.resolve({
            data: msg.data,
            res: p.res,
            halfExtent: msg.halfExtent,
            peak: msg.peak,
          });
        }
      });

      worker.addEventListener('error', (ev) => {
        const p = this.pending;
        this.pending = null;
        if (this.worker === worker) {
          worker.terminate();
          this.worker = null;
        }
        if (p) p.reject(ev);
      });

      const req: BakeRequest = {
        type: 'requestBake',
        scene: { orbital: { n: params.n, l: params.l, m: params.m } },
        res: params.res,
      };
      worker.postMessage(req);
    });
  }

  cancel(): void {
    const p = this.pending;
    const w = this.worker;
    this.pending = null;
    this.worker = null;
    if (w) w.terminate();
    if (p) p.reject(new BakeCancelledError());
  }

  dispose(): void {
    this.cancel();
  }
}
