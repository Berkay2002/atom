import { describe, expect, it } from 'vitest';

import {
  DEFAULT_STORED_STATE,
  parseStoredState,
  serializeStoredState,
  type StoredState,
} from './storage';

describe('parseStoredState', () => {
  it('returns null for missing input so the page falls back to defaults', () => {
    expect(parseStoredState(null)).toBeNull();
    expect(parseStoredState(undefined)).toBeNull();
  });

  it('returns null for non-JSON input', () => {
    expect(parseStoredState('not json {')).toBeNull();
  });

  it('returns null for JSON that is not an object', () => {
    expect(parseStoredState('42')).toBeNull();
    expect(parseStoredState('null')).toBeNull();
    expect(parseStoredState('"string"')).toBeNull();
  });

  it('round-trips a clean stored state unchanged', () => {
    const s: StoredState = {
      elementZ: 6,
      n: 4,
      l: 2,
      m: -1,
      colormap: 'VIRIDIS',
      autoRotate: true,
      hudVisible: false,
      useBareZ: true,
    };
    expect(parseStoredState(serializeStoredState(s))).toEqual(s);
  });

  it('clamps elementZ to [1, 18] and falls back for missing/invalid values', () => {
    const tooLow = parseStoredState(JSON.stringify({ ...DEFAULT_STORED_STATE, elementZ: 0 }))!;
    expect(tooLow.elementZ).toBe(1);
    const tooHigh = parseStoredState(JSON.stringify({ ...DEFAULT_STORED_STATE, elementZ: 99 }))!;
    expect(tooHigh.elementZ).toBe(18);
    const missing = parseStoredState(JSON.stringify({ n: 2, l: 1, m: 0 }))!;
    expect(missing.elementZ).toBe(DEFAULT_STORED_STATE.elementZ);
    const garbled = parseStoredState(JSON.stringify({ ...DEFAULT_STORED_STATE, elementZ: 'foo' }))!;
    expect(garbled.elementZ).toBe(DEFAULT_STORED_STATE.elementZ);
  });

  it('clamps l down when it exceeds n - 1', () => {
    // n=1 only permits l=0; stored l=5 must clamp.
    const raw = JSON.stringify({
      n: 1,
      l: 5,
      m: 0,
      colormap: 'INFERNO',
      autoRotate: false,
      hudVisible: true,
    });
    const parsed = parseStoredState(raw)!;
    expect(parsed.n).toBe(1);
    expect(parsed.l).toBe(0);
    expect(parsed.m).toBe(0);
  });

  it('clamps |m| > l back into [-l, l]', () => {
    // l=1 permits m ∈ {-1, 0, 1}; stored m=7 must clamp to 1.
    const raw = JSON.stringify({
      n: 2,
      l: 1,
      m: 7,
      colormap: 'INFERNO',
      autoRotate: false,
      hudVisible: true,
    });
    const parsed = parseStoredState(raw)!;
    expect(parsed.m).toBe(1);
  });

  it('clamps n into [1, 6]', () => {
    const tooLow = parseStoredState(JSON.stringify({ ...DEFAULT_STORED_STATE, n: -3, l: 0, m: 0 }))!;
    expect(tooLow.n).toBe(1);

    const tooHigh = parseStoredState(JSON.stringify({ ...DEFAULT_STORED_STATE, n: 99, l: 0, m: 0 }))!;
    expect(tooHigh.n).toBe(6);
  });

  it('falls back to INFERNO for unknown colormap names', () => {
    const raw = JSON.stringify({ ...DEFAULT_STORED_STATE, colormap: 'NOT_A_REAL_MAP' });
    expect(parseStoredState(raw)!.colormap).toBe('INFERNO');
  });

  it('replaces non-boolean autoRotate/hudVisible/useBareZ with defaults', () => {
    const raw = JSON.stringify({
      ...DEFAULT_STORED_STATE,
      autoRotate: 'yes',
      hudVisible: 0,
      useBareZ: 'true',
    });
    const parsed = parseStoredState(raw)!;
    expect(parsed.autoRotate).toBe(DEFAULT_STORED_STATE.autoRotate);
    expect(parsed.hudVisible).toBe(DEFAULT_STORED_STATE.hudVisible);
    expect(parsed.useBareZ).toBe(DEFAULT_STORED_STATE.useBareZ);
  });

  it('preserves a true useBareZ flag', () => {
    const raw = JSON.stringify({ ...DEFAULT_STORED_STATE, useBareZ: true });
    expect(parseStoredState(raw)!.useBareZ).toBe(true);
  });

  it('defaults useBareZ to false when missing from the stored blob', () => {
    const raw = JSON.stringify({ n: 2, l: 1, m: 0 });
    expect(parseStoredState(raw)!.useBareZ).toBe(false);
  });

  it('replaces NaN/Infinity in n/l/m with defaults before clamping', () => {
    const raw = JSON.stringify({
      ...DEFAULT_STORED_STATE,
      n: 'foo',
      l: null,
      m: false,
    });
    const parsed = parseStoredState(raw)!;
    expect(parsed.n).toBe(DEFAULT_STORED_STATE.n);
    expect(parsed.l).toBe(DEFAULT_STORED_STATE.l);
  });
});
