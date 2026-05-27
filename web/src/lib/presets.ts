// Named (label, n, l, m) presets for the quick-jump chip strip.
//
// Mirrors `PRESETS` in `crates/atom-desktop/src/ui.rs` verbatim. The labels
// use unicode glyphs (`²`, `³`) since the web HUD can render them directly,
// whereas the desktop uses ASCII (`^2`, `^3`) due to egui font constraints.
// Values for (n, l, m) are byte-identical.

export type Preset = {
  label: string;
  n: number;
  l: number;
  m: number;
};

export const PRESETS: readonly Preset[] = [
  { label: '1s',           n: 1, l: 0, m:  0 },
  { label: '2s',           n: 2, l: 0, m:  0 },
  { label: '2p_x',         n: 2, l: 1, m:  1 },
  { label: '2p_y',         n: 2, l: 1, m: -1 },
  { label: '2p_z',         n: 2, l: 1, m:  0 },
  { label: '3d_xy',        n: 3, l: 2, m: -2 },
  { label: '3d_xz',        n: 3, l: 2, m:  1 },
  { label: '3d_yz',        n: 3, l: 2, m: -1 },
  { label: '3d_(x²-y²)',   n: 3, l: 2, m:  2 },
  { label: '3d_(z²)',      n: 3, l: 2, m:  0 },
  { label: '4f_(z³)',      n: 4, l: 3, m:  0 },
];
