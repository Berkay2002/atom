// Tests for the pure URL-string helpers in `scene-url.ts`. The codec
// itself (encodeScene / decodeScene) is covered exhaustively by Rust unit
// tests in `atom-core::scene::codec_tests` — we don't re-test it here
// because pulling the WASM module into jsdom adds bootstrap noise without
// catching any TS-side bug the helpers below don't already exercise.

import { describe, expect, it } from 'vitest';

import { readSceneParam, SCENE_PARAM_NAME, withSceneParam } from './scene-url';

describe('readSceneParam', () => {
  it('returns null for empty / missing inputs', () => {
    expect(readSceneParam(null)).toBeNull();
    expect(readSceneParam(undefined)).toBeNull();
    expect(readSceneParam('')).toBeNull();
  });

  it('returns null when the scene param is absent', () => {
    expect(readSceneParam('?foo=bar')).toBeNull();
    expect(readSceneParam('foo=bar')).toBeNull();
  });

  it('extracts the scene value with or without a leading ?', () => {
    expect(readSceneParam(`?${SCENE_PARAM_NAME}=v1:1/1/0/0/eff/0/1.00`)).toBe(
      'v1:1/1/0/0/eff/0/1.00',
    );
    expect(readSceneParam(`${SCENE_PARAM_NAME}=v1:6/2/1/0/eff/3/1.20`)).toBe(
      'v1:6/2/1/0/eff/3/1.20',
    );
  });

  it('returns null for an explicitly empty scene value', () => {
    // `?s=` produces a present-but-empty value. Treat it as absent so the
    // page falls back cleanly to localStorage / defaults.
    expect(readSceneParam(`?${SCENE_PARAM_NAME}=`)).toBeNull();
  });

  it('coexists with other search params', () => {
    expect(readSceneParam(`?other=x&${SCENE_PARAM_NAME}=v1:8/2/1/0/bare/0/1.00&z=y`)).toBe(
      'v1:8/2/1/0/bare/0/1.00',
    );
  });
});

describe('withSceneParam', () => {
  it('writes the scene param into an empty search string', () => {
    expect(withSceneParam('', 'v1:1/1/0/0/eff/0/1.00')).toBe(
      `${SCENE_PARAM_NAME}=v1%3A1%2F1%2F0%2F0%2Feff%2F0%2F1.00`,
    );
  });

  it('preserves unrelated existing params', () => {
    const next = withSceneParam('?other=hello', 'v1:1/1/0/0/eff/0/1.00');
    const decoded = new URLSearchParams(next);
    expect(decoded.get('other')).toBe('hello');
    expect(decoded.get(SCENE_PARAM_NAME)).toBe('v1:1/1/0/0/eff/0/1.00');
  });

  it('replaces an existing scene param rather than appending', () => {
    const next = withSceneParam(
      `?${SCENE_PARAM_NAME}=v1:1/1/0/0/eff/0/1.00`,
      'v1:6/2/1/0/eff/3/1.20',
    );
    const decoded = new URLSearchParams(next);
    // Only one value present, not two — the URL writer is for replace, not
    // append, so the address bar can't grow unbounded over time.
    expect(decoded.getAll(SCENE_PARAM_NAME)).toEqual(['v1:6/2/1/0/eff/3/1.20']);
  });
});
