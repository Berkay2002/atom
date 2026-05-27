import { describe, expect, it } from 'vitest';

import { buildColormapLutBytes, raymarchIntensity } from './raymarch-contract';
import { FRAG_SRC } from './shaders';

function collapseWhitespace(source: string): string {
  return source.replace(/\s+/g, ' ').trim();
}

describe('raymarchIntensity', () => {
  it('returns zero for zero accumulated density', () => {
    expect(raymarchIntensity(0, 5, 1)).toBe(0);
  });

  it('matches the exponential tone mapping for a representative sum', () => {
    const sum = 1;
    const k = 1;
    const exposure = 1;

    expect(raymarchIntensity(sum, k, exposure)).toBeCloseTo(1 - Math.exp(-1), 12);
  });

  it('scales by exposure and clamps to one', () => {
    expect(raymarchIntensity(10, 5, 2)).toBe(1);
  });

  it('grows as k increases for the same sum and exposure', () => {
    const low = raymarchIntensity(0.5, 1, 1);
    const high = raymarchIntensity(0.5, 4, 1);

    expect(high).toBeGreaterThan(low);
  });
});

describe('raymarch shader contract', () => {
  it('keeps the fragment tone-mapping expression in the shader source', () => {
    const shader = collapseWhitespace(FRAG_SRC);

    expect(shader).toMatch(
      /float intensity = clamp\(exposure \* \(1\.0 - exp\(-k \* sum\)\), 0\.0, 1\.0\);/,
    );
  });
});

describe('buildColormapLutBytes', () => {
  it('interpolates endpoints, midpoint, and alpha across 256 entries', () => {
    const lut = buildColormapLutBytes([
      [0, 0, 0],
      [255, 0, 0],
    ]);

    expect(lut).toHaveLength(256 * 4);
    expect(Array.from(lut.slice(0, 4))).toEqual([0, 0, 0, 255]);
    expect(Array.from(lut.slice(128 * 4, 128 * 4 + 4))).toEqual([128, 0, 0, 255]);
    expect(Array.from(lut.slice(255 * 4, 255 * 4 + 4))).toEqual([255, 0, 0, 255]);
  });

  it('switches stops cleanly at a multi-stop boundary', () => {
    const lut = buildColormapLutBytes([
      [0, 0, 0],
      [64, 0, 0],
      [64, 128, 0],
      [255, 128, 0],
    ]);

    expect(Array.from(lut.slice(84 * 4, 84 * 4 + 4))).toEqual([63, 0, 0, 255]);
    expect(Array.from(lut.slice(85 * 4, 85 * 4 + 4))).toEqual([64, 0, 0, 255]);
    expect(Array.from(lut.slice(86 * 4, 86 * 4 + 4))).toEqual([64, 2, 0, 255]);
  });

  it('rejects a colormap with fewer than two stops', () => {
    expect(() => buildColormapLutBytes([[1, 2, 3]])).toThrow(
      'colormap needs at least 2 stops, got 1',
    );
  });
});
