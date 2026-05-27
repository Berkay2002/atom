// @vitest-environment jsdom

import { describe, expect, it } from 'vitest';

import { BakeClient } from './client';

class FakeWorker extends EventTarget {
  posted: unknown[] = [];
  terminated = false;

  postMessage(message: unknown): void {
    this.posted.push(message);
  }

  terminate(): void {
    this.terminated = true;
  }
}

describe('BakeClient', () => {
  it('posts a Scene-shaped bake request to the worker', () => {
    const worker = new FakeWorker();
    const client = new BakeClient(() => worker as unknown as Worker);

    void client.requestBake({
      elementZ: 6,
      n: 2,
      l: 1,
      m: 0,
      useBareZ: true,
      res: 96,
    });

    expect(worker.posted).toEqual([
      {
        type: 'requestBake',
        scene: {
          atom: {
            elementZ: 6,
            orbital: { n: 2, l: 1, m: 0 },
          },
          view: { useBareZ: true },
        },
        res: 96,
      },
    ]);
  });
});
