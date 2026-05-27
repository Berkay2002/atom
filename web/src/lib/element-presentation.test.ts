import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import {
  ELEMENT_PRESENTATIONS,
  elementPresentationFor,
  homoFor,
} from './element-presentation';
import init, { element_presentation } from '../../wasm/atom_core.js';

describe('element presentation projection', () => {
  beforeAll(async () => {
    await init(await readFile(new URL('../../wasm/atom_core_bg.wasm', import.meta.url)));
  });

  it('matches the shared Rust element presentation facts', () => {
    // This TS snapshot is a projection of `atom_core::element_presentation`.
    // The parity check below compares it directly to the wasm-exported
    // Rust projection so the snapshot cannot drift silently.
    expect(ELEMENT_PRESENTATIONS).toHaveLength(18);

    for (const entry of ELEMENT_PRESENTATIONS) {
      const actual = element_presentation(entry.atomicNumber);
      expect(actual, `Z=${entry.atomicNumber}`).toBeDefined();
      expect(actual?.atomic_number).toBe(entry.atomicNumber);
      expect(actual?.symbol).toBe(entry.symbol);
      expect(actual?.display_name).toBe(entry.displayName);
      expect(actual?.config_text).toBe(entry.configText);
      expect(actual?.homo_n).toBe(entry.homo.n);
      expect(actual?.homo_l).toBe(entry.homo.l);
      expect(actual?.homo_m).toBe(entry.homo.m);
      expect(actual?.slot_period).toBe(entry.slot.period);
      expect(actual?.slot_group).toBe(entry.slot.group);
    }
  });

  it('looks up a presentation by atomic number and returns null out of range', () => {
    expect(element_presentation(0)).toBeUndefined();
    expect(element_presentation(19)).toBeUndefined();
    expect(elementPresentationFor(1)?.symbol).toBe('H');
    expect(elementPresentationFor(6)?.displayName).toBe('Carbon');
    expect(elementPresentationFor(18)?.configText).toBe('[Ne] 3s² 3p⁶');
    expect(elementPresentationFor(0)).toBeNull();
    expect(elementPresentationFor(19)).toBeNull();
    expect(elementPresentationFor(-5)).toBeNull();
  });

  it('falls back to 1s for unsupported HOMO lookups', () => {
    expect(homoFor(0)).toEqual({ n: 1, l: 0, m: 0 });
    expect(homoFor(19)).toEqual({ n: 1, l: 0, m: 0 });
    expect(homoFor(-5)).toEqual({ n: 1, l: 0, m: 0 });
  });
});
