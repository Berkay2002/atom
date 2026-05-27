// Highest occupied orbital (HOMO) for each element Z=1..18.
//
// Mirrors `atom_core::homo` on the Rust side: when the user picks an
// element in the periodic-table chip grid, we snap `(n, l, m)` to that
// element's HOMO so the canvas always shows the element's iconic
// orbital instead of leaving the previous selection in place. That
// previous-selection behaviour produced empty renders whenever the
// orbital was unoccupied at the new Z under Slater shielding (the
// classic case: leaving 3d_xz selected and switching to Carbon
// silently clamps z_eff to 0 and the cloud disappears).
//
// `m = 0` everywhere — the textbook "vertical" choice that's always
// in-range and produces the p_z dumbbell / d_z² lobe / spherical s
// that most chemistry diagrams use.
//
// We duplicate the table here (rather than calling out to the WASM
// `homo` export) because the chip-click handler runs synchronously and
// the WASM bootstrap is async on the page hydrate path. The
// `homo_matches_rust_table` parity test in `wasm_parity.rs` keeps the
// two tables in sync.

export type Homo = { n: number; l: number; m: number };

// Z=1..18 in index 0..17. Derived from each element's last
// `(n, l, count)` electron-config entry in
// `crates/atom-core/src/element.rs`.
const HOMO_TABLE: readonly Homo[] = [
  { n: 1, l: 0, m: 0 }, // 1  H  — 1s
  { n: 1, l: 0, m: 0 }, // 2  He — 1s
  { n: 2, l: 0, m: 0 }, // 3  Li — 2s
  { n: 2, l: 0, m: 0 }, // 4  Be — 2s
  { n: 2, l: 1, m: 0 }, // 5  B  — 2p_z
  { n: 2, l: 1, m: 0 }, // 6  C  — 2p_z
  { n: 2, l: 1, m: 0 }, // 7  N  — 2p_z
  { n: 2, l: 1, m: 0 }, // 8  O  — 2p_z
  { n: 2, l: 1, m: 0 }, // 9  F  — 2p_z
  { n: 2, l: 1, m: 0 }, // 10 Ne — 2p_z
  { n: 3, l: 0, m: 0 }, // 11 Na — 3s
  { n: 3, l: 0, m: 0 }, // 12 Mg — 3s
  { n: 3, l: 1, m: 0 }, // 13 Al — 3p_z
  { n: 3, l: 1, m: 0 }, // 14 Si — 3p_z
  { n: 3, l: 1, m: 0 }, // 15 P  — 3p_z
  { n: 3, l: 1, m: 0 }, // 16 S  — 3p_z
  { n: 3, l: 1, m: 0 }, // 17 Cl — 3p_z
  { n: 3, l: 1, m: 0 }, // 18 Ar — 3p_z
];

/**
 * HOMO for the given atomic number. Falls back to 1s for any Z outside
 * the 1..=18 range we support — same fallback the bake uses, so the
 * canvas never goes blank from a bogus Z value.
 */
export function homoFor(z: number): Homo {
  const idx = Math.round(z) - 1;
  if (idx < 0 || idx >= HOMO_TABLE.length) {
    return { n: 1, l: 0, m: 0 };
  }
  return HOMO_TABLE[idx];
}
