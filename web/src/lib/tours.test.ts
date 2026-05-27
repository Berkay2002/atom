// Tests for the pure parts of `tours.ts` — parser + URL-param helpers.
// The `loadTour` / `loadTourIndex` fetch wrappers are exercised by the
// page-level integration; here we only verify the parse and URL logic
// that has branches worth covering.

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  clearTourParams,
  parseIndex,
  parseTour,
  readTourParams,
  TOUR_SLUG_PARAM,
  TOUR_STEP_PARAM,
  TourLoadError,
  withTourParams,
} from './tours';

const TOURS_DIR = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '..',
  '..',
  'public',
  'tours',
);

function readJson(name: string): unknown {
  return JSON.parse(readFileSync(resolve(TOURS_DIR, name), 'utf-8'));
}

describe('parseIndex', () => {
  it('parses a valid index', () => {
    const entries = parseIndex({
      tours: [
        { slug: 'a', name: 'A', description: 'first' },
        { slug: 'b', name: 'B', description: 'second' },
      ],
    });
    expect(entries).toEqual([
      { slug: 'a', name: 'A', description: 'first' },
      { slug: 'b', name: 'B', description: 'second' },
    ]);
  });

  it('throws when `tours` is missing or not an array', () => {
    expect(() => parseIndex(null)).toThrow(TourLoadError);
    expect(() => parseIndex({})).toThrow(TourLoadError);
    expect(() => parseIndex({ tours: 'nope' })).toThrow(TourLoadError);
  });

  it('throws on a malformed entry', () => {
    expect(() => parseIndex({ tours: [{ slug: 'a' }] })).toThrow(/name/);
    expect(() => parseIndex({ tours: [{ slug: '', name: 'x', description: '' }] })).toThrow(/slug/);
  });
});

describe('parseTour', () => {
  it('parses a valid tour and stamps the slug', () => {
    const tour = parseTour('demo', {
      name: 'Demo',
      description: 'desc',
      steps: [
        { scene: 'v1:1/1/0/0/eff/0/1.00', caption: 'first' },
        { scene: 'v1:2/1/0/0/eff/0/1.00', caption: 'second' },
      ],
    });
    expect(tour.slug).toBe('demo');
    expect(tour.name).toBe('Demo');
    expect(tour.steps).toHaveLength(2);
    expect(tour.steps[0].scene).toBe('v1:1/1/0/0/eff/0/1.00');
  });

  it('rejects empty / missing steps', () => {
    expect(() => parseTour('x', { name: 'x', description: '', steps: [] })).toThrow(/steps/);
    expect(() => parseTour('x', { name: 'x', description: '' })).toThrow(/steps/);
  });

  it('rejects a step missing scene or caption', () => {
    expect(() =>
      parseTour('x', {
        name: 'x',
        description: '',
        steps: [{ scene: '', caption: 'hi' }],
      }),
    ).toThrow(/scene/);
    expect(() =>
      parseTour('x', {
        name: 'x',
        description: '',
        steps: [{ scene: 'v1:1/1/0/0/eff/0/1.00' }],
      }),
    ).toThrow(/caption/);
  });

  it('rejects a non-string `name`', () => {
    expect(() => parseTour('x', { name: 42, description: '', steps: [{ scene: 's', caption: 'c' }] }))
      .toThrow(/name/);
  });
});

describe('readTourParams', () => {
  it('returns null when params absent', () => {
    expect(readTourParams(null)).toBeNull();
    expect(readTourParams('')).toBeNull();
    expect(readTourParams('?other=x')).toBeNull();
  });

  it('parses tour + step (1-indexed in URL, 0-indexed in return)', () => {
    expect(readTourParams(`?${TOUR_SLUG_PARAM}=demo&${TOUR_STEP_PARAM}=3`)).toEqual({
      slug: 'demo',
      step: 2,
    });
  });

  it('defaults step to 1 (= index 0) when only the slug is given', () => {
    expect(readTourParams(`?${TOUR_SLUG_PARAM}=demo`)).toEqual({ slug: 'demo', step: 0 });
  });

  it('returns null on non-positive / non-numeric step', () => {
    expect(readTourParams(`?${TOUR_SLUG_PARAM}=demo&${TOUR_STEP_PARAM}=0`)).toBeNull();
    expect(readTourParams(`?${TOUR_SLUG_PARAM}=demo&${TOUR_STEP_PARAM}=-1`)).toBeNull();
    expect(readTourParams(`?${TOUR_SLUG_PARAM}=demo&${TOUR_STEP_PARAM}=abc`)).toBeNull();
  });
});

describe('withTourParams', () => {
  it('sets the tour and step (1-indexed) and drops any existing `s`', () => {
    const p = withTourParams('?s=v1:1/1/0/0/eff/0/1.00&other=y', 'demo', 2);
    expect(p.get(TOUR_SLUG_PARAM)).toBe('demo');
    expect(p.get(TOUR_STEP_PARAM)).toBe('3');
    expect(p.get('s')).toBeNull();
    expect(p.get('other')).toBe('y');
  });
});

describe('clearTourParams', () => {
  it('removes tour params while keeping everything else', () => {
    const p = clearTourParams(`?${TOUR_SLUG_PARAM}=demo&${TOUR_STEP_PARAM}=2&s=v1:1/1/0/0/eff/0/1.00`);
    expect(p.get(TOUR_SLUG_PARAM)).toBeNull();
    expect(p.get(TOUR_STEP_PARAM)).toBeNull();
    expect(p.get('s')).toBe('v1:1/1/0/0/eff/0/1.00');
  });
});

// Smoke check the shipped tour content. Catches typos in the JSON files
// — a malformed scene string or missing caption would have made it past
// `npm run build` (these are static assets, not compiled) but break the
// app at runtime when a user clicks the tour.
describe('shipped tours', () => {
  it('index.json parses and lists the expected slugs', () => {
    const entries = parseIndex(readJson('index.json'));
    const slugs = entries.map((e) => e.slug).sort();
    expect(slugs).toEqual(['orbital-shapes', 'shielding-period-2']);
    entries.forEach((e) => {
      expect(e.name.length).toBeGreaterThan(0);
      expect(e.description.length).toBeGreaterThan(0);
    });
  });

  it('shielding-period-2.json parses with valid-looking scene strings', () => {
    const tour = parseTour('shielding-period-2', readJson('shielding-period-2.json'));
    expect(tour.steps).toHaveLength(5);
    // Every step's scene must start with the v1 prefix; the full decode
    // round-trip is covered by atom-core's Rust tests.
    tour.steps.forEach((s) => {
      expect(s.scene.startsWith('v1:')).toBe(true);
      // Captions live in a fixed-height bar; keep them readable.
      expect(s.caption.length).toBeLessThan(400);
    });
  });

  it('orbital-shapes.json parses with valid-looking scene strings', () => {
    const tour = parseTour('orbital-shapes', readJson('orbital-shapes.json'));
    expect(tour.steps).toHaveLength(7);
    tour.steps.forEach((s) => {
      expect(s.scene.startsWith('v1:')).toBe(true);
      expect(s.caption.length).toBeLessThan(400);
    });
  });
});
