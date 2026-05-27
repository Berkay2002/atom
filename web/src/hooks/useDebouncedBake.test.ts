// @vitest-environment jsdom

import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { BakeCancelledError, type IBakeClient, type Volume } from '@/lib/bake/client';
import { useDebouncedBake } from './useDebouncedBake';

type Deferred<T> = {
  promise: Promise<T>;
  resolve: (v: T) => void;
  reject: (e: unknown) => void;
};

function deferred<T>(): Deferred<T> {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

/** Hand-rolled BakeClient stand-in. Each requestBake() returns a fresh
 * deferred promise that the test resolves explicitly. */
class MockBakeClient implements IBakeClient {
  readonly bakes: Array<{ params: unknown; def: Deferred<Volume> }> = [];
  cancel = vi.fn(() => {
    // Match the real BakeClient: cancelling an in-flight bake rejects
    // its promise with BakeCancelledError.
    const last = this.bakes[this.bakes.length - 1];
    if (last && last.def) {
      // Only reject if still pending — best-effort, harmless if already settled.
      last.def.reject(new BakeCancelledError());
    }
  });
  dispose = vi.fn();
  requestBake = vi.fn((params: unknown) => {
    const def = deferred<Volume>();
    this.bakes.push({ params, def });
    return def.promise;
  });
}

function makeVolume(seed: number): Volume {
  return {
    data: new Float32Array([seed]),
    res: 96,
    halfExtent: seed,
    peak: 1,
  };
}

describe('useDebouncedBake', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it('fires exactly one bake 150ms after a single param change', async () => {
    const client = new MockBakeClient();
    const { rerender } = renderHook(
      ({ p }: { p: { elementZ: number; n: number; l: number; m: number; useBareZ: boolean; res: number } }) =>
        useDebouncedBake(p, client),
      { initialProps: { p: { elementZ: 1, n: 3, l: 2, m: 1, useBareZ: false, res: 96 } } },
    );

    // Initial mount also schedules a bake — wait it out for clarity.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(150);
    });
    expect(client.requestBake).toHaveBeenCalledTimes(1);

    rerender({ p: { elementZ: 1, n: 4, l: 2, m: 1, useBareZ: false, res: 96 } });
    expect(client.requestBake).toHaveBeenCalledTimes(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(149);
    });
    expect(client.requestBake).toHaveBeenCalledTimes(1);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });
    expect(client.requestBake).toHaveBeenCalledTimes(2);
    expect(client.requestBake).toHaveBeenLastCalledWith({ elementZ: 1, n: 4, l: 2, m: 1, useBareZ: false, res: 96 });
  });

  it('coalesces two rapid changes into a single bake with the latest params', async () => {
    const client = new MockBakeClient();
    const { rerender } = renderHook(
      ({ p }: { p: { elementZ: number; n: number; l: number; m: number; useBareZ: boolean; res: number } }) =>
        useDebouncedBake(p, client),
      { initialProps: { p: { elementZ: 1, n: 3, l: 2, m: 1, useBareZ: false, res: 96 } } },
    );

    // Drain the initial mount bake.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(150);
    });
    client.requestBake.mockClear();

    rerender({ p: { elementZ: 1, n: 4, l: 2, m: 1, useBareZ: false, res: 96 } });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(50);
    });
    expect(client.requestBake).not.toHaveBeenCalled();

    rerender({ p: { elementZ: 1, n: 5, l: 2, m: 1, useBareZ: false, res: 96 } });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(149);
    });
    expect(client.requestBake).not.toHaveBeenCalled();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });
    expect(client.requestBake).toHaveBeenCalledTimes(1);
    expect(client.requestBake).toHaveBeenLastCalledWith({ elementZ: 1, n: 5, l: 2, m: 1, useBareZ: false, res: 96 });
  });

  it('supersedes an in-flight bake when params change again', async () => {
    const client = new MockBakeClient();
    const { result, rerender } = renderHook(
      ({ p }: { p: { elementZ: number; n: number; l: number; m: number; useBareZ: boolean; res: number } }) =>
        useDebouncedBake(p, client),
      { initialProps: { p: { elementZ: 1, n: 3, l: 2, m: 1, useBareZ: false, res: 96 } } },
    );

    // First bake is dispatched but never resolved.
    await act(async () => {
      await vi.advanceTimersByTimeAsync(150);
    });
    expect(client.requestBake).toHaveBeenCalledTimes(1);
    expect(result.current.baking).toBe(true);

    // Change params while the first bake is still pending.
    rerender({ p: { elementZ: 1, n: 4, l: 2, m: 1, useBareZ: false, res: 96 } });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(150);
    });
    expect(client.requestBake).toHaveBeenCalledTimes(2);
    expect(client.requestBake).toHaveBeenLastCalledWith({ elementZ: 1, n: 4, l: 2, m: 1, useBareZ: false, res: 96 });

    // Resolving the second bake should land in state; the first never does.
    const secondVolume = makeVolume(2);
    await act(async () => {
      client.bakes[1].def.resolve(secondVolume);
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(result.current.volume).toBe(secondVolume);

    // Late-arriving first-bake resolution must NOT clobber state.
    const firstVolume = makeVolume(1);
    await act(async () => {
      client.bakes[0].def.resolve(firstVolume);
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(result.current.volume).toBe(secondVolume);
  });

  it('toggles baking=true at request start and false at result delivery', async () => {
    const client = new MockBakeClient();
    const { result } = renderHook(() =>
      useDebouncedBake({ elementZ: 1, n: 3, l: 2, m: 1, useBareZ: false, res: 96 }, client),
    );

    expect(result.current.baking).toBe(false);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(150);
    });
    expect(client.requestBake).toHaveBeenCalledTimes(1);
    expect(result.current.baking).toBe(true);

    await act(async () => {
      client.bakes[0].def.resolve(makeVolume(42));
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(result.current.baking).toBe(false);
    expect(result.current.volume?.halfExtent).toBe(42);
  });
});
