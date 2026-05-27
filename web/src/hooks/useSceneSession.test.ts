// @vitest-environment jsdom

import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { DEFAULT_STORED_STATE, STORAGE_KEY, serializeStoredState } from '@/lib/storage';
import { loadTour, type Tour } from '@/lib/tours';
import { URL_DEBOUNCE_MS, STORAGE_DEBOUNCE_MS, useSceneSession } from './useSceneSession';

vi.mock('@/lib/scene-url', () => {
  class MockSceneDecodeError extends Error {
    constructor(message: string) {
      super(message);
      this.name = 'SceneDecodeError';
    }
  }

  return {
    SCENE_PARAM_NAME: 's',
    SceneDecodeError: MockSceneDecodeError,
    loadCodec: vi.fn(() => Promise.resolve()),
    readSceneParam: vi.fn((searchString: string | null | undefined) => {
      if (!searchString) return null;
      const stripped = searchString.startsWith('?') ? searchString.slice(1) : searchString;
      const params = new URLSearchParams(stripped);
      const raw = params.get('s');
      return raw && raw.length > 0 ? raw : null;
    }),
    decodeScene: vi.fn((encoded: string) => {
      if (encoded === 'malformed') throw new MockSceneDecodeError('bad scene');
      if (encoded === 'url-scene') {
        return {
          elementZ: 8,
          n: 2,
          l: 1,
          m: -1,
          useBareZ: true,
          colormap: 'VIRIDIS',
          exposure: 1,
        };
      }
      if (encoded === 'tour-step-one') {
        return {
          elementZ: 3,
          n: 2,
          l: 0,
          m: 0,
          useBareZ: false,
          colormap: 'INFERNO',
          exposure: 1,
        };
      }
      if (encoded === 'tour-step-two') {
        return {
          elementZ: 9,
          n: 4,
          l: 2,
          m: -1,
          useBareZ: true,
          colormap: 'TURBO',
          exposure: 1,
        };
      }
      throw new MockSceneDecodeError(`unknown scene ${encoded}`);
    }),
    encodeScene: vi.fn((state) =>
      [
        'encoded',
        state.elementZ,
        state.n,
        state.l,
        state.m,
        state.useBareZ ? 'bare' : 'eff',
        state.colormap,
      ].join('-'),
    ),
  };
});

vi.mock('@/lib/tours', () => {
  const TOUR_SLUG_PARAM = 'tour';
  const TOUR_STEP_PARAM = 'step';

  const demoTour = {
    slug: 'demo',
    name: 'Demo tour',
    description: 'Demo tour description',
    steps: [
      { scene: 'tour-step-one', caption: 'First stop' },
      { scene: 'tour-step-two', caption: 'Second stop' },
    ],
  };

  return {
    TOUR_SLUG_PARAM,
    TOUR_STEP_PARAM,
    loadTour: vi.fn((slug: string) => {
      if (slug !== 'demo') return Promise.reject(new Error(`unknown tour ${slug}`));
      return Promise.resolve(demoTour);
    }),
    readTourParams: vi.fn((searchString: string | null | undefined) => {
      if (!searchString) return null;
      const stripped = searchString.startsWith('?') ? searchString.slice(1) : searchString;
      const params = new URLSearchParams(stripped);
      const slug = params.get(TOUR_SLUG_PARAM);
      const stepRaw = params.get(TOUR_STEP_PARAM);
      if (!slug) return null;
      const stepNum = stepRaw ? Number.parseInt(stepRaw, 10) : 1;
      if (!Number.isFinite(stepNum) || stepNum < 1) return null;
      return { slug, step: stepNum - 1 };
    }),
    withTourParams: vi.fn((currentSearch: string | null | undefined, slug: string, step: number) => {
      const stripped = currentSearch?.startsWith('?')
        ? currentSearch.slice(1)
        : currentSearch ?? '';
      const params = new URLSearchParams(stripped);
      params.set(TOUR_SLUG_PARAM, slug);
      params.set(TOUR_STEP_PARAM, String(step + 1));
      params.delete('s');
      return params;
    }),
    clearTourParams: vi.fn((currentSearch: string | null | undefined) => {
      const stripped = currentSearch?.startsWith('?')
        ? currentSearch.slice(1)
        : currentSearch ?? '';
      const params = new URLSearchParams(stripped);
      params.delete(TOUR_SLUG_PARAM);
      params.delete(TOUR_STEP_PARAM);
      return params;
    }),
  };
});

function stored(overrides: Partial<typeof DEFAULT_STORED_STATE> = {}) {
  return {
    ...DEFAULT_STORED_STATE,
    elementZ: 6,
    n: 3,
    l: 2,
    m: 1,
    colormap: 'PLASMA' as const,
    autoRotate: true,
    hudVisible: false,
    useBareZ: false,
    ...overrides,
  };
}

async function flushPromises() {
  await Promise.resolve();
  await Promise.resolve();
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

function demoTour(): Tour {
  return {
    slug: 'demo',
    name: 'Demo tour',
    description: 'Demo tour description',
    steps: [
      { scene: 'tour-step-one', caption: 'First stop' },
      { scene: 'tour-step-two', caption: 'Second stop' },
    ],
  };
}

function routerReplacingLocation() {
  return {
    replace: vi.fn((href: string) => {
      window.history.replaceState(null, '', href);
    }),
  };
}

describe('useSceneSession', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    window.localStorage.clear();
    window.history.replaceState(null, '', '/');
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it('hydrates normal mode with Scene URL before stored state', async () => {
    window.localStorage.setItem(STORAGE_KEY, serializeStoredState(stored()));
    window.history.replaceState(null, '', '/?s=url-scene');

    const router = { replace: vi.fn() };
    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: 's=url-scene',
        router,
      }),
    );

    await act(flushPromises);

    expect(result.current.elementZ).toBe(8);
    expect(result.current.params).toEqual({ n: 2, l: 1, m: -1 });
    expect(result.current.colormap).toBe('VIRIDIS');
    expect(result.current.useBareZ).toBe(true);
    expect(result.current.autoRotate).toBe(true);
    expect(result.current.hudVisible).toBe(false);
    expect(result.current.decodeError).toBeNull();
  });

  it('hydrates normal mode from stored state before defaults when no Scene URL exists', async () => {
    window.localStorage.setItem(STORAGE_KEY, serializeStoredState(stored({ elementZ: 7 })));

    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: '',
        router: { replace: vi.fn() },
      }),
    );

    await act(flushPromises);

    expect(result.current.elementZ).toBe(7);
    expect(result.current.params).toEqual({ n: 3, l: 2, m: 1 });
    expect(result.current.colormap).toBe('PLASMA');
    expect(result.current.autoRotate).toBe(true);
    expect(result.current.hudVisible).toBe(false);
    expect(result.current.useBareZ).toBe(false);
  });

  it('reports malformed Scene URLs and falls back without crashing', async () => {
    window.history.replaceState(null, '', '/?s=malformed');

    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: 's=malformed',
        router: { replace: vi.fn() },
      }),
    );

    await act(flushPromises);

    expect(result.current.elementZ).toBe(DEFAULT_STORED_STATE.elementZ);
    expect(result.current.params).toEqual({
      n: DEFAULT_STORED_STATE.n,
      l: DEFAULT_STORED_STATE.l,
      m: DEFAULT_STORED_STATE.m,
    });
    expect(result.current.decodeError).toBe("Couldn't load shared view: bad scene. Using defaults.");
  });

  it('exposes page-facing intent handlers for normal control changes', async () => {
    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: '',
        router: { replace: vi.fn() },
      }),
    );

    await act(flushPromises);

    act(() => {
      result.current.handleOrbitalChange({ n: 4, l: 2, m: -1 });
      result.current.handleColormapChange('MAGMA');
      result.current.handleUseBareZChange(true);
      result.current.handleAutoRotateChange(true);
    });

    expect(result.current.params).toEqual({ n: 4, l: 2, m: -1 });
    expect(result.current.colormap).toBe('MAGMA');
    expect(result.current.useBareZ).toBe(true);
    expect(result.current.autoRotate).toBe(true);
  });

  it('debounces normal Scene URL writes to the latest mutation', async () => {
    const router = { replace: vi.fn() };
    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: '',
        router,
      }),
    );

    await act(async () => {
      await flushPromises();
      await vi.advanceTimersByTimeAsync(URL_DEBOUNCE_MS);
    });
    router.replace.mockClear();

    act(() => {
      result.current.handleElementChange(2);
      result.current.handleOrbitalChange({ n: 2, l: 0, m: 0 });
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(URL_DEBOUNCE_MS - 1);
    });
    expect(router.replace).not.toHaveBeenCalled();

    act(() => {
      result.current.handleElementChange(10);
      result.current.handleOrbitalChange({ n: 3, l: 1, m: 0 });
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(URL_DEBOUNCE_MS);
    });

    expect(router.replace).toHaveBeenCalledTimes(1);
    expect(router.replace).toHaveBeenLastCalledWith('/?s=encoded-10-3-1-0-eff-INFERNO', {
      scroll: false,
    });
  });

  it('debounces normal local storage writes to the latest mutation', async () => {
    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: '',
        router: { replace: vi.fn() },
      }),
    );

    await act(async () => {
      await flushPromises();
      await vi.advanceTimersByTimeAsync(STORAGE_DEBOUNCE_MS);
    });
    window.localStorage.clear();

    act(() => {
      result.current.handleAutoRotateChange(true);
      result.current.handleElementChange(4);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(STORAGE_DEBOUNCE_MS - 1);
    });
    expect(window.localStorage.getItem(STORAGE_KEY)).toBeNull();

    act(() => {
      result.current.handleAutoRotateChange(false);
      result.current.handleElementChange(12);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(STORAGE_DEBOUNCE_MS);
    });

    expect(JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? '{}')).toMatchObject({
      elementZ: 12,
      autoRotate: false,
      hudVisible: true,
    });
  });

  it('persists HUD visibility changes through the normal storage debounce', async () => {
    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: '',
        router: { replace: vi.fn() },
      }),
    );

    await act(async () => {
      await flushPromises();
      await vi.advanceTimersByTimeAsync(STORAGE_DEBOUNCE_MS);
    });
    window.localStorage.clear();

    act(() => {
      result.current.toggleHud();
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(STORAGE_DEBOUNCE_MS);
    });

    expect(JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? '{}')).toMatchObject({
      hudVisible: false,
    });
  });

  it('hydrates Tour URLs before normal Scene URLs and stored state', async () => {
    window.localStorage.setItem(STORAGE_KEY, serializeStoredState(stored({ elementZ: 6 })));
    window.history.replaceState(null, '', '/?tour=demo&step=2&s=url-scene');

    const router = routerReplacingLocation();
    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: 'tour=demo&step=2&s=url-scene',
        router,
      }),
    );

    await act(flushPromises);

    expect(loadTour).toHaveBeenCalledWith('demo');
    expect(result.current.tourMode?.tour.slug).toBe('demo');
    expect(result.current.tourMode?.stepIndex).toBe(1);
    expect(result.current.currentStep?.caption).toBe('Second stop');
    expect(result.current.elementZ).toBe(9);
    expect(result.current.params).toEqual({ n: 4, l: 2, m: -1 });
    expect(result.current.colormap).toBe('TURBO');
    expect(result.current.useBareZ).toBe(true);
    expect(result.current.autoRotate).toBe(true);
    expect(result.current.hudVisible).toBe(false);
    expect(router.replace).toHaveBeenLastCalledWith('/?tour=demo&step=2', { scroll: false });
  });

  it('navigates Tour steps, applies the step Scene, and writes Tour URL params without Scene params', async () => {
    window.history.replaceState(null, '', '/?tour=demo&step=1&s=url-scene');

    const router = routerReplacingLocation();
    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: 'tour=demo&step=1&s=url-scene',
        router,
      }),
    );
    await act(flushPromises);
    router.replace.mockClear();

    act(() => {
      result.current.handleTourNext();
    });
    await act(flushPromises);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(URL_DEBOUNCE_MS);
    });

    expect(result.current.tourMode?.stepIndex).toBe(1);
    expect(result.current.elementZ).toBe(9);
    expect(result.current.params).toEqual({ n: 4, l: 2, m: -1 });
    expect(router.replace).toHaveBeenCalledTimes(1);
    expect(router.replace).toHaveBeenLastCalledWith('/?tour=demo&step=2', { scroll: false });
  });

  it('suppresses normal storage writes while Tour mode is active', async () => {
    window.history.replaceState(null, '', '/?tour=demo&step=1');

    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: 'tour=demo&step=1',
        router: routerReplacingLocation(),
      }),
    );
    await act(flushPromises);
    window.localStorage.clear();

    act(() => {
      result.current.handleAutoRotateChange(true);
      result.current.handleElementChange(12);
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(STORAGE_DEBOUNCE_MS);
    });

    expect(window.localStorage.getItem(STORAGE_KEY)).toBeNull();
  });

  it('restores normal Scene URL persistence after exiting a Tour', async () => {
    window.history.replaceState(null, '', '/?tour=demo&step=2&other=keep');

    const router = routerReplacingLocation();
    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: 'tour=demo&step=2&other=keep',
        router,
      }),
    );
    await act(flushPromises);
    router.replace.mockClear();

    act(() => {
      result.current.handleTourExit();
    });
    await act(flushPromises);

    expect(result.current.tourMode).toBeNull();
    expect(router.replace).toHaveBeenCalledWith('/?other=keep', { scroll: false });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(URL_DEBOUNCE_MS);
    });

    expect(router.replace).toHaveBeenLastCalledWith(
      '/?other=keep&s=encoded-9-4-2--1-bare-TURBO',
      { scroll: false },
    );
  });

  it('ignores a stale Tour URL load that resolves after Tour exit', async () => {
    const pendingTour = deferred<Tour>();
    vi.mocked(loadTour).mockReturnValueOnce(pendingTour.promise);
    window.history.replaceState(null, '', '/?tour=demo&step=2&other=keep');

    const router = routerReplacingLocation();
    const { result } = renderHook(() =>
      useSceneSession({
        initialSearch: 'tour=demo&step=2&other=keep',
        router,
      }),
    );
    await act(flushPromises);

    act(() => {
      result.current.handleTourExit();
    });
    await act(flushPromises);

    await act(async () => {
      pendingTour.resolve(demoTour());
      await flushPromises();
    });

    expect(result.current.tourMode).toBeNull();
    expect(result.current.elementZ).toBe(DEFAULT_STORED_STATE.elementZ);
    expect(router.replace).toHaveBeenCalledWith('/?other=keep', { scroll: false });
    expect(router.replace).not.toHaveBeenCalledWith('/?tour=demo&step=2', { scroll: false });
  });
});
