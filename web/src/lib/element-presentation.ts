// Web projection of the shared element presentation facts from
// `atom_core::element_presentation`.
//
// The browser still needs synchronous access for click handlers and CSS
// placement, so the UI consumes this projected snapshot instead of
// waiting for the WASM bootstrap on every interaction. Keep this table in
// atomic-number order and in sync with the shared Rust facts.

export type Homo = { n: number; l: number; m: number };

export type PeriodicSlot = { period: number; group: number };

export type ElementPresentation = {
  atomicNumber: number;
  symbol: string;
  displayName: string;
  configText: string;
  homo: Homo;
  slot: PeriodicSlot;
};

// Projection snapshot of `atom_core::element_presentation` for H-Ar.
// This is intentionally data-only: web controls render from these facts,
// but the shared Rust crate owns the actual chemistry logic.
export const ELEMENT_PRESENTATIONS: readonly ElementPresentation[] = [
  {
    atomicNumber: 1,
    symbol: 'H',
    displayName: 'Hydrogen',
    configText: '1s¹',
    homo: { n: 1, l: 0, m: 0 },
    slot: { period: 1, group: 1 },
  },
  {
    atomicNumber: 2,
    symbol: 'He',
    displayName: 'Helium',
    configText: '1s²',
    homo: { n: 1, l: 0, m: 0 },
    slot: { period: 1, group: 18 },
  },
  {
    atomicNumber: 3,
    symbol: 'Li',
    displayName: 'Lithium',
    configText: '1s² 2s¹',
    homo: { n: 2, l: 0, m: 0 },
    slot: { period: 2, group: 1 },
  },
  {
    atomicNumber: 4,
    symbol: 'Be',
    displayName: 'Beryllium',
    configText: '1s² 2s²',
    homo: { n: 2, l: 0, m: 0 },
    slot: { period: 2, group: 2 },
  },
  {
    atomicNumber: 5,
    symbol: 'B',
    displayName: 'Boron',
    configText: '1s² 2s² 2p¹',
    homo: { n: 2, l: 1, m: 0 },
    slot: { period: 2, group: 13 },
  },
  {
    atomicNumber: 6,
    symbol: 'C',
    displayName: 'Carbon',
    configText: '1s² 2s² 2p²',
    homo: { n: 2, l: 1, m: 0 },
    slot: { period: 2, group: 14 },
  },
  {
    atomicNumber: 7,
    symbol: 'N',
    displayName: 'Nitrogen',
    configText: '1s² 2s² 2p³',
    homo: { n: 2, l: 1, m: 0 },
    slot: { period: 2, group: 15 },
  },
  {
    atomicNumber: 8,
    symbol: 'O',
    displayName: 'Oxygen',
    configText: '1s² 2s² 2p⁴',
    homo: { n: 2, l: 1, m: 0 },
    slot: { period: 2, group: 16 },
  },
  {
    atomicNumber: 9,
    symbol: 'F',
    displayName: 'Fluorine',
    configText: '1s² 2s² 2p⁵',
    homo: { n: 2, l: 1, m: 0 },
    slot: { period: 2, group: 17 },
  },
  {
    atomicNumber: 10,
    symbol: 'Ne',
    displayName: 'Neon',
    configText: '1s² 2s² 2p⁶',
    homo: { n: 2, l: 1, m: 0 },
    slot: { period: 2, group: 18 },
  },
  {
    atomicNumber: 11,
    symbol: 'Na',
    displayName: 'Sodium',
    configText: '[Ne] 3s¹',
    homo: { n: 3, l: 0, m: 0 },
    slot: { period: 3, group: 1 },
  },
  {
    atomicNumber: 12,
    symbol: 'Mg',
    displayName: 'Magnesium',
    configText: '[Ne] 3s²',
    homo: { n: 3, l: 0, m: 0 },
    slot: { period: 3, group: 2 },
  },
  {
    atomicNumber: 13,
    symbol: 'Al',
    displayName: 'Aluminium',
    configText: '[Ne] 3s² 3p¹',
    homo: { n: 3, l: 1, m: 0 },
    slot: { period: 3, group: 13 },
  },
  {
    atomicNumber: 14,
    symbol: 'Si',
    displayName: 'Silicon',
    configText: '[Ne] 3s² 3p²',
    homo: { n: 3, l: 1, m: 0 },
    slot: { period: 3, group: 14 },
  },
  {
    atomicNumber: 15,
    symbol: 'P',
    displayName: 'Phosphorus',
    configText: '[Ne] 3s² 3p³',
    homo: { n: 3, l: 1, m: 0 },
    slot: { period: 3, group: 15 },
  },
  {
    atomicNumber: 16,
    symbol: 'S',
    displayName: 'Sulfur',
    configText: '[Ne] 3s² 3p⁴',
    homo: { n: 3, l: 1, m: 0 },
    slot: { period: 3, group: 16 },
  },
  {
    atomicNumber: 17,
    symbol: 'Cl',
    displayName: 'Chlorine',
    configText: '[Ne] 3s² 3p⁵',
    homo: { n: 3, l: 1, m: 0 },
    slot: { period: 3, group: 17 },
  },
  {
    atomicNumber: 18,
    symbol: 'Ar',
    displayName: 'Argon',
    configText: '[Ne] 3s² 3p⁶',
    homo: { n: 3, l: 1, m: 0 },
    slot: { period: 3, group: 18 },
  },
] as const;

export function elementPresentationFor(z: number): ElementPresentation | null {
  const idx = Math.round(z) - 1;
  if (idx < 0 || idx >= ELEMENT_PRESENTATIONS.length) {
    return null;
  }
  return ELEMENT_PRESENTATIONS[idx];
}

/**
 * HOMO for the given atomic number. Falls back to 1s for any Z outside
 * the H-Ar projection range, preserving the old web behavior.
 */
export function homoFor(z: number): Homo {
  return elementPresentationFor(z)?.homo ?? { n: 1, l: 0, m: 0 };
}
