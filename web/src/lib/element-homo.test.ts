import { describe, expect, it } from 'vitest';

import { homoFor } from './element-homo';

describe('homoFor', () => {
  it('returns the textbook HOMO for each H–Ar element', () => {
    // Locked against the same table tested in atom-core's
    // `homo_matches_textbook_homo_for_h_through_ar` — drift between the
    // TS table and the Rust table would silently desync the desktop
    // and web "snap on element change" behaviour.
    const cases: [number, number, number][] = [
      [1, 1, 0],
      [2, 1, 0],
      [3, 2, 0],
      [4, 2, 0],
      [5, 2, 1],
      [6, 2, 1],
      [7, 2, 1],
      [8, 2, 1],
      [9, 2, 1],
      [10, 2, 1],
      [11, 3, 0],
      [12, 3, 0],
      [13, 3, 1],
      [14, 3, 1],
      [15, 3, 1],
      [16, 3, 1],
      [17, 3, 1],
      [18, 3, 1],
    ];
    for (const [z, n, l] of cases) {
      expect(homoFor(z), `Z=${z}`).toEqual({ n, l, m: 0 });
    }
  });

  it('falls back to 1s for out-of-range Z', () => {
    expect(homoFor(0)).toEqual({ n: 1, l: 0, m: 0 });
    expect(homoFor(19)).toEqual({ n: 1, l: 0, m: 0 });
    expect(homoFor(-5)).toEqual({ n: 1, l: 0, m: 0 });
  });
});
