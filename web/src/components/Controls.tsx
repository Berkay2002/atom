'use client';

// n/l/m chip strips with the hydrogen quantum-number bounds:
//   n ∈ [1..6], l ∈ [0..n-1], m ∈ [-l..l].
//
// The component only renders chips for *valid* options, so the user
// physically can't pick an invalid (n, l, m). The clampOrbital helper is
// still applied on every emit — it matters when n or l decreases and the
// previously-selected l or m falls out of the new range.
//
// Visual style mirrors the desktop HUD card (see
// `crates/atom-desktop/src/ui_tokens.rs` and `ui_widgets.rs::card_frame` /
// `chip_strip`): a dark translucent "glass card" anchored top-left, with
// a small uppercase label above each row of rounded chips. Tokens are
// ported as CSS variables on the panel so the chip styles can reference
// them without duplicating the alpha values.

import {
  useState,
  useSyncExternalStore,
  type CSSProperties,
  type PointerEvent,
} from 'react';

import {
  COLORMAP_LABELS,
  COLORMAP_ORDER,
  COLORMAPS,
  type ColormapName,
  type ColormapStops,
} from '@/lib/colormaps';
import { PRESETS, type Preset } from '@/lib/presets';
import { useSceneCaption } from '@/lib/scene-caption';

export type OrbitalParams = {
  n: number;
  l: number;
  m: number;
};

/** Symbols for the first 18 elements, indexed by `atomicNumber - 1`. */
export const ELEMENT_SYMBOLS: readonly string[] = [
  'H', 'He', 'Li', 'Be', 'B', 'C',
  'N', 'O', 'F', 'Ne', 'Na', 'Mg',
  'Al', 'Si', 'P', 'S', 'Cl', 'Ar',
];

export type ControlsProps = {
  /** Selected element's atomic number (1..=18). */
  elementZ: number;
  onElementChange: (next: number) => void;
  value: OrbitalParams;
  onChange: (next: OrbitalParams) => void;
  colormap: ColormapName;
  onColormapChange: (next: ColormapName) => void;
  autoRotate: boolean;
  onAutoRotateChange: (next: boolean) => void;
  /** Bare-Z toggle. `false` (default) uses Slater-screened `z_eff`;
   *  `true` uses the bare atomic number `Z` so users can see the
   *  shielding effect by toggling between the two. */
  useBareZ: boolean;
  onUseBareZChange: (next: boolean) => void;
};

const N_MIN = 1;
const N_MAX = 6;

function clampOrbital(n: number, l: number, m: number): OrbitalParams {
  const nc = Math.min(N_MAX, Math.max(N_MIN, Math.round(n)));
  const lc = Math.min(nc - 1, Math.max(0, Math.round(l)));
  const mc = Math.min(lc, Math.max(-lc, Math.round(m)));
  return { n: nc, l: lc, m: mc };
}

// Tokens ported from `crates/atom-desktop/src/ui_tokens.rs`. The desktop
// uses egui Color32 (rgba bytes); these are the same colors expressed as
// CSS rgba(). Keep them in sync if the desktop palette ever shifts.
const TOKEN = {
  cardBg: 'rgba(14, 12, 18, 0.85)',
  accent: '#ff8a4c',
  accentDim: 'rgba(255, 138, 76, 0.30)',
  textPrimary: 'rgba(255, 255, 255, 0.85)',
  textTertiary: 'rgba(255, 255, 255, 0.35)',
  surfaceMute: 'rgba(255, 255, 255, 0.05)',
  surfaceMuteHover: 'rgba(255, 255, 255, 0.09)',
  border: 'rgba(255, 255, 255, 0.06)',
  divider: 'rgba(255, 255, 255, 0.08)',
  radiusCard: 10,
  radiusChip: 999, // pill chips for the web restyle (desktop uses 4px on bool chips)
  bodySize: 12,
  labelSize: 9,
  edgeInset: 16,
} as const;

const panelStyle: CSSProperties = {
  position: 'fixed',
  top: TOKEN.edgeInset,
  left: TOKEN.edgeInset,
  zIndex: 10,
  display: 'flex',
  flexDirection: 'column',
  gap: 10,
  // Cap card width so the preset chip strip (`flexWrap: 'wrap'` in
  // `stripStyle`) actually wraps to multiple rows instead of stretching
  // the card to fit all 11 presets in one line. 280px comfortably fits
  // the first row of short presets (1s, 2s, 2p_x, 2p_y, 2p_z) and lets
  // the wider 3d_* / 4f_* chips flow onto subsequent rows.
  maxWidth: 280,
  // 12px horizontal / 14px vertical matches card_frame's symmetric(12, 14)
  // inner margin in ui_widgets.rs.
  padding: '14px 12px',
  borderRadius: TOKEN.radiusCard,
  background: TOKEN.cardBg,
  borderStyle: 'solid',
  borderWidth: 1,
  borderColor: TOKEN.border,
  color: TOKEN.textPrimary,
  font: `${TOKEN.bodySize}px / 1.3 system-ui, -apple-system, "Segoe UI", sans-serif`,
  backdropFilter: 'blur(12px) saturate(140%)',
  WebkitBackdropFilter: 'blur(12px) saturate(140%)',
  boxShadow: '0 8px 24px rgba(0, 0, 0, 0.35)',
  pointerEvents: 'auto',
  userSelect: 'none',
};

const rowStyle: CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  gap: 4,
};

const labelStyle: CSSProperties = {
  fontSize: TOKEN.labelSize,
  // Desktop labels are uppercase small-caps with 0.16em tracking.
  textTransform: 'uppercase',
  letterSpacing: '0.16em',
  color: TOKEN.textTertiary,
  fontWeight: 500,
  // Standard "hoverable glossary term" affordance: dotted underline +
  // help cursor signals to first-time visitors that hovering the label
  // reveals an explanation tooltip. The underline color matches the dim
  // label color so it reads as a subtle hint, not a link.
  cursor: 'help',
  textDecoration: 'underline dotted',
  textDecorationColor: TOKEN.textTertiary,
  textUnderlineOffset: '3px',
};

const stripStyle: CSSProperties = {
  display: 'flex',
  flexWrap: 'wrap',
  gap: 4,
};

// Split the border into its three longhand properties so React doesn't
// warn about mixing `border` shorthand with `borderColor` longhand across
// rerenders. Selected/unselected both write all three keys, so the
// reconciler never sees a property "disappear".
const chipBase: CSSProperties = {
  minWidth: 26,
  padding: '4px 10px',
  borderRadius: TOKEN.radiusChip,
  font: 'inherit',
  fontVariantNumeric: 'tabular-nums',
  lineHeight: 1.1,
  cursor: 'pointer',
  borderStyle: 'solid',
  borderWidth: 1,
  borderColor: 'transparent',
  background: TOKEN.surfaceMute,
  color: TOKEN.textPrimary,
  transition: 'background 120ms ease, border-color 120ms ease, color 120ms ease',
};

const chipHover: CSSProperties = {
  ...chipBase,
  background: TOKEN.surfaceMuteHover,
};

const chipSelected: CSSProperties = {
  ...chipBase,
  background: TOKEN.accentDim,
  borderColor: TOKEN.accent,
  color: TOKEN.textPrimary,
};

const dividerStyle: CSSProperties = {
  border: 'none',
  borderTop: `1px solid ${TOKEN.divider}`,
  margin: '2px 0',
};

// "What am I looking at" caption (issue 06). A small, lightly-styled
// paragraph slot beneath the element picker so visitors who don't read
// spectroscopic notation can ground themselves before touching the
// chips. The text comes from `atom-core::caption` via WASM — the same
// source the desktop renders, so the two targets never drift.
//
// `whiteSpace: 'normal'` lets the line wrap on narrow viewports; on
// wide cards the caption typically stays one line. `minHeight` reserves
// vertical space so the layout doesn't reflow when the WASM module
// resolves and the caption pops in for the first time.
const captionStyle: CSSProperties = {
  font: 'inherit',
  fontSize: 11,
  lineHeight: 1.4,
  color: TOKEN.textPrimary,
  opacity: 0.75,
  margin: 0,
  // Two lines of vertical space — covers the long descriptions (e.g.
  // 3d cloverleaf) without leaving a huge gap for one-liners like 1s.
  minHeight: '2.6em',
};

// Compact swatch — height kept short so the 2×3 grid stays vertically
// dense; width is driven by the grid cell (`width: '100%'`).
const SWATCH_H = 16;

const swatchRowStyle: CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(3, 1fr)',
  gap: 4,
};

const swatchBase: CSSProperties = {
  width: '100%',
  height: SWATCH_H,
  borderRadius: 4,
  cursor: 'pointer',
  padding: 0,
  borderStyle: 'solid',
  borderWidth: 1,
  borderColor: 'transparent',
  transition: 'border-color 120ms ease, transform 120ms ease',
};

const swatchSelected: CSSProperties = {
  ...swatchBase,
  borderColor: TOKEN.accent,
};

function range(lo: number, hi: number): number[] {
  const out: number[] = [];
  for (let i = lo; i <= hi; i += 1) out.push(i);
  return out;
}

// Native-title tooltips so non-physicist visitors can hover any chip or
// row label and learn what the quantum number means. Browser-native
// `title` attribute — no library, no positioning logic.
const LABEL_TOOLTIPS: Record<string, string> = {
  element: 'Element — picking a higher-Z atom shrinks the cloud (Slater shielding)',
  n: 'Principal quantum number — energy level / size of the orbital',
  l: 'Orbital angular momentum — shape of the orbital (0=s, 1=p, 2=d, 3=f)',
  m: 'Magnetic quantum number — orientation of the orbital in space (−l ≤ m ≤ +l)',
  colormap: 'Color palette applied to the density',
  presets: 'Common named orbitals',
  autoRotate: 'Spin the camera around the orbital (~12s per revolution). Drag still works on top.',
  bareZ: "Bare Z removes electron shielding to show what the orbital would look like if the nucleus's full charge reached the electron.",
};

const N_TOOLTIPS: Record<number, string> = {
  1: 'K shell — innermost',
  2: 'L shell',
  3: 'M shell',
  4: 'N shell',
  5: 'O shell',
  6: 'P shell',
};

const L_TOOLTIPS: Record<number, string> = {
  0: 's — spherical',
  1: 'p — dumbbell',
  2: 'd — cloverleaf',
  3: 'f — complex',
  4: 'g',
  5: 'h',
};

const mTooltip = (v: number) => `m = ${v}`;

type ChipStripProps = {
  label: string;
  values: number[];
  selected: number;
  onPick: (v: number) => void;
  idPrefix: string;
  labelTooltip?: string;
  chipTooltip?: (v: number) => string | undefined;
};

function ChipStrip({
  label,
  values,
  selected,
  onPick,
  idPrefix,
  labelTooltip,
  chipTooltip,
}: ChipStripProps) {
  const [hovered, setHovered] = useState<number | null>(null);
  return (
    <div style={rowStyle}>
      <span style={labelStyle} title={labelTooltip}>
        {label}
      </span>
      <div style={stripStyle} role="radiogroup" aria-label={label}>
        {values.map((v) => {
          const isSelected = v === selected;
          const isHovered = !isSelected && hovered === v;
          const style = isSelected ? chipSelected : isHovered ? chipHover : chipBase;
          return (
            <button
              key={v}
              type="button"
              role="radio"
              aria-checked={isSelected}
              id={`${idPrefix}-${v}`}
              onClick={() => onPick(v)}
              onPointerEnter={() => setHovered(v)}
              onPointerLeave={() => setHovered((h) => (h === v ? null : h))}
              style={style}
              title={chipTooltip?.(v)}
            >
              {v}
            </button>
          );
        })}
      </div>
    </div>
  );
}

// Periodic-table-shaped element picker (issue 05). The 18 elements of
// periods 1-3 are laid out at their real periodic-table positions: H in
// column 1 and He in column 18; Li, Be in columns 1-2 with B..Ne jumping
// to columns 13-18; same shape for period 3. The gaps where the
// transition metals would live are intentional — the layout itself is
// part of the pedagogy.
//
// `row` is 1-indexed and `col` is the CSS grid column (1..=18).
type PeriodicCell = {
  z: number;
  symbol: string;
  row: number;
  col: number;
};

const PERIODIC_LAYOUT: readonly PeriodicCell[] = [
  // Period 1
  { z: 1, symbol: 'H', row: 1, col: 1 },
  { z: 2, symbol: 'He', row: 1, col: 18 },
  // Period 2
  { z: 3, symbol: 'Li', row: 2, col: 1 },
  { z: 4, symbol: 'Be', row: 2, col: 2 },
  { z: 5, symbol: 'B', row: 2, col: 13 },
  { z: 6, symbol: 'C', row: 2, col: 14 },
  { z: 7, symbol: 'N', row: 2, col: 15 },
  { z: 8, symbol: 'O', row: 2, col: 16 },
  { z: 9, symbol: 'F', row: 2, col: 17 },
  { z: 10, symbol: 'Ne', row: 2, col: 18 },
  // Period 3
  { z: 11, symbol: 'Na', row: 3, col: 1 },
  { z: 12, symbol: 'Mg', row: 3, col: 2 },
  { z: 13, symbol: 'Al', row: 3, col: 13 },
  { z: 14, symbol: 'Si', row: 3, col: 14 },
  { z: 15, symbol: 'P', row: 3, col: 15 },
  { z: 16, symbol: 'S', row: 3, col: 16 },
  { z: 17, symbol: 'Cl', row: 3, col: 17 },
  { z: 18, symbol: 'Ar', row: 3, col: 18 },
];

// Narrow-viewport breakpoint: below this width the 18-column layout would
// produce sub-tappable cells (~14px each at 280px panel width). We fall
// back to the issue-02 6-cols × 3-rows dense grid so the picker remains
// usable on mobile-portrait. The threshold is in CSS pixels and is
// evaluated via a `matchMedia` listener in `ElementPicker`.
const PERIODIC_NARROW_BREAKPOINT_PX = 400;

// Each periodic cell is square; the 18 columns share equal fractions of
// the row. The card itself caps at 280px (see `panelStyle.maxWidth`), so
// individual cells are ~15px wide at the cap. We shrink fonts/padding
// down from the regular chip to keep the symbols legible.
const periodicGridStyle: CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(18, 1fr)',
  gap: 2,
};

const periodicCellBase: CSSProperties = {
  // Cells are tiny when squeezed into 18 columns; remove the chip's pill
  // radius and minWidth so they stay square and don't overflow the row.
  minWidth: 0,
  padding: '2px 0',
  borderRadius: 3,
  font: 'inherit',
  fontSize: 10,
  fontWeight: 600,
  lineHeight: 1.1,
  cursor: 'pointer',
  borderStyle: 'solid',
  borderWidth: 1,
  borderColor: 'transparent',
  background: TOKEN.surfaceMute,
  color: TOKEN.textPrimary,
  transition: 'background 120ms ease, border-color 120ms ease, color 120ms ease',
  textAlign: 'center',
  // Force square aspect so the grid reads like a real periodic table
  // rather than a strip of wide rectangles.
  aspectRatio: '1 / 1',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
};

const periodicCellHover: CSSProperties = {
  ...periodicCellBase,
  background: TOKEN.surfaceMuteHover,
};

const periodicCellSelected: CSSProperties = {
  ...periodicCellBase,
  background: TOKEN.accentDim,
  borderColor: TOKEN.accent,
};

// Mobile-portrait fallback: 6×3 dense grid (the issue-02 shape), reusing
// the same chip styling as the n/l/m strips. We render the same
// PERIODIC_LAYOUT entries but without the explicit `gridColumn`.
const denseGridStyle: CSSProperties = {
  display: 'grid',
  gridTemplateColumns: 'repeat(6, 1fr)',
  gap: 4,
};

type ElementPickerProps = {
  value: number;
  onPick: (z: number) => void;
};

// Subscribe to a `matchMedia` query the React-19-idiomatic way: this
// avoids the `react-hooks/set-state-in-effect` warning we'd hit with a
// useEffect + setState pair. `useSyncExternalStore` is purpose-built for
// reading values from external mutable sources (browser APIs, window
// dimensions) without cascading renders.
const NARROW_MEDIA_QUERY = `(max-width: ${PERIODIC_NARROW_BREAKPOINT_PX}px)`;

function subscribeNarrowMQ(onChange: () => void): () => void {
  if (typeof window === 'undefined') return () => {};
  const mq = window.matchMedia(NARROW_MEDIA_QUERY);
  mq.addEventListener('change', onChange);
  return () => mq.removeEventListener('change', onChange);
}

function getNarrowSnapshot(): boolean {
  return window.matchMedia(NARROW_MEDIA_QUERY).matches;
}

// SSR snapshot — render the periodic layout on the server so the markup
// is identical for desktop and most viewports. The client snapshot kicks
// in immediately after hydration and switches to the dense layout if
// needed.
function getNarrowServerSnapshot(): boolean {
  return false;
}

function ElementPicker({ value, onPick }: ElementPickerProps) {
  const [hovered, setHovered] = useState<number | null>(null);
  const isNarrow = useSyncExternalStore(
    subscribeNarrowMQ,
    getNarrowSnapshot,
    getNarrowServerSnapshot,
  );

  const gridStyle = isNarrow ? denseGridStyle : periodicGridStyle;

  return (
    <div style={rowStyle}>
      <span style={labelStyle} title={LABEL_TOOLTIPS.element}>
        element
      </span>
      <div style={gridStyle} role="radiogroup" aria-label="element">
        {PERIODIC_LAYOUT.map(({ z, symbol, row, col }) => {
          const isSelected = z === value;
          const isHovered = !isSelected && hovered === z;
          const baseStyle = isNarrow ? chipBase : periodicCellBase;
          const hoverStyle = isNarrow ? chipHover : periodicCellHover;
          const selectedStyle = isNarrow ? chipSelected : periodicCellSelected;
          const style: CSSProperties = isSelected
            ? selectedStyle
            : isHovered
              ? hoverStyle
              : baseStyle;
          // Only the periodic layout cares about explicit row/column
          // placement; the dense fallback fills cells in source order.
          const placedStyle: CSSProperties = isNarrow
            ? style
            : { ...style, gridRow: row, gridColumn: col };
          return (
            <button
              key={symbol}
              type="button"
              role="radio"
              aria-checked={isSelected}
              aria-label={symbol}
              onClick={() => onPick(z)}
              onPointerEnter={() => setHovered(z)}
              onPointerLeave={() => setHovered((h) => (h === z ? null : h))}
              style={placedStyle}
              title={`${symbol} — Z = ${z}`}
            >
              {symbol}
            </button>
          );
        })}
      </div>
    </div>
  );
}

// Build a CSS linear-gradient string from the 8 colormap stops. CSS
// interpolates the gradient itself between adjacent color-stops the same
// way the renderer's LUT does — perceptually identical at this size.
function gradientCss(stops: ColormapStops): string {
  const parts = stops.map((c, i) => {
    const pct = (i / (stops.length - 1)) * 100;
    return `rgb(${c[0]}, ${c[1]}, ${c[2]}) ${pct.toFixed(2)}%`;
  });
  return `linear-gradient(to right, ${parts.join(', ')})`;
}

type ColormapPickerProps = {
  value: ColormapName;
  onPick: (next: ColormapName) => void;
};

function ColormapPicker({ value, onPick }: ColormapPickerProps) {
  return (
    <div style={rowStyle}>
      <span style={labelStyle} title={LABEL_TOOLTIPS.colormap}>
        colormap
      </span>
      <div style={swatchRowStyle} role="radiogroup" aria-label="colormap">
        {COLORMAP_ORDER.map((name) => {
          const isSelected = name === value;
          const style: CSSProperties = {
            ...(isSelected ? swatchSelected : swatchBase),
            background: gradientCss(COLORMAPS[name]),
          };
          return (
            <button
              key={name}
              type="button"
              role="radio"
              aria-checked={isSelected}
              aria-label={COLORMAP_LABELS[name]}
              title={COLORMAP_LABELS[name]}
              onClick={() => onPick(name)}
              style={style}
            />
          );
        })}
      </div>
    </div>
  );
}

// Compact toggle switch styled to match the glass card: a small pill
// track with a sliding knob. Mirrors `toggle_switch` in
// `ui_widgets.rs` (~26×14 px desktop) — the web version is sized for a
// slightly larger touch target.
const TOGGLE_W = 30;
const TOGGLE_H = 16;
const TOGGLE_PAD = 2;

const toggleRowStyle: CSSProperties = {
  ...rowStyle,
  flexDirection: 'row',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: 8,
};

function toggleTrackStyle(on: boolean): CSSProperties {
  return {
    position: 'relative',
    width: TOGGLE_W,
    height: TOGGLE_H,
    borderRadius: TOGGLE_H / 2,
    background: on ? TOKEN.accentDim : TOKEN.surfaceMute,
    borderStyle: 'solid',
    borderWidth: 1,
    borderColor: on ? TOKEN.accent : TOKEN.border,
    transition: 'background 120ms ease, border-color 120ms ease',
    cursor: 'pointer',
    flexShrink: 0,
    padding: 0,
  };
}

function toggleKnobStyle(on: boolean): CSSProperties {
  const knobSize = TOGGLE_H - TOGGLE_PAD * 2 - 2; // -2 for the 1px border on each side
  return {
    position: 'absolute',
    top: TOGGLE_PAD,
    left: on ? TOGGLE_W - knobSize - TOGGLE_PAD - 2 : TOGGLE_PAD,
    width: knobSize,
    height: knobSize,
    borderRadius: '50%',
    background: on ? TOKEN.accent : TOKEN.textPrimary,
    transition: 'left 140ms ease, background 120ms ease',
  };
}

type ToggleRowProps = {
  label: string;
  labelTooltip?: string;
  value: boolean;
  onChange: (next: boolean) => void;
};

function ToggleRow({ label, labelTooltip, value, onChange }: ToggleRowProps) {
  return (
    <div style={toggleRowStyle}>
      <span style={labelStyle} title={labelTooltip}>
        {label}
      </span>
      <button
        type="button"
        role="switch"
        aria-checked={value}
        aria-label={label}
        onClick={() => onChange(!value)}
        style={toggleTrackStyle(value)}
      >
        <span style={toggleKnobStyle(value)} />
      </button>
    </div>
  );
}

type PresetStripProps = {
  value: OrbitalParams;
  onPick: (p: Preset) => void;
};

function matchesPreset(p: Preset, v: OrbitalParams): boolean {
  return p.n === v.n && p.l === v.l && p.m === v.m;
}

function PresetStrip({ value, onPick }: PresetStripProps) {
  const [hovered, setHovered] = useState<number | null>(null);
  return (
    <div style={rowStyle}>
      <span style={labelStyle} title={LABEL_TOOLTIPS.presets}>
        presets
      </span>
      <div style={stripStyle} role="radiogroup" aria-label="presets">
        {PRESETS.map((p, i) => {
          const isSelected = matchesPreset(p, value);
          const isHovered = !isSelected && hovered === i;
          const style = isSelected ? chipSelected : isHovered ? chipHover : chipBase;
          return (
            <button
              key={p.label}
              type="button"
              role="radio"
              aria-checked={isSelected}
              onClick={() => onPick(p)}
              onPointerEnter={() => setHovered(i)}
              onPointerLeave={() => setHovered((h) => (h === i ? null : h))}
              style={style}
              title={`n=${p.n}, l=${p.l}, m=${p.m}`}
            >
              {p.label}
            </button>
          );
        })}
      </div>
    </div>
  );
}

type SceneCaptionProps = {
  elementZ: number;
  value: OrbitalParams;
};

// Plain-language caption slot (issue 06). Sits between the periodic
// picker and the bare-Z toggle, where a visitor's eye lands after
// they pick an element. The string itself is composed by
// `atom-core::caption` and surfaced via `useSceneCaption`; until the
// WASM module is ready the slot renders empty (the parent reserves
// vertical space via `minHeight` so the layout doesn't reflow).
function SceneCaption({ elementZ, value }: SceneCaptionProps) {
  const caption = useSceneCaption({
    elementZ,
    n: value.n,
    l: value.l,
    m: value.m,
  });
  return (
    <p style={captionStyle} aria-live="polite">
      {caption ?? ''}
    </p>
  );
}

export default function Controls({
  elementZ,
  onElementChange,
  value,
  onChange,
  colormap,
  onColormapChange,
  autoRotate,
  onAutoRotateChange,
  useBareZ,
  onUseBareZChange,
}: ControlsProps) {
  // Keep pointer events from bubbling into the canvas drag/zoom handlers.
  const stop = (e: PointerEvent<HTMLDivElement>) => {
    e.stopPropagation();
  };

  const emit = (n: number, l: number, m: number) => {
    onChange(clampOrbital(n, l, m));
  };

  const nValues = range(N_MIN, N_MAX);
  const lValues = range(0, value.n - 1);
  const mValues = range(-value.l, value.l);

  return (
    <div
      style={panelStyle}
      onPointerDown={stop}
      onPointerMove={stop}
      onPointerUp={stop}
      onWheel={(e) => e.stopPropagation()}
    >
      <ElementPicker value={elementZ} onPick={onElementChange} />
      <SceneCaption elementZ={elementZ} value={value} />
      <ToggleRow
        label={useBareZ ? 'bare Z' : 'effective Z'}
        labelTooltip={LABEL_TOOLTIPS.bareZ}
        value={useBareZ}
        onChange={onUseBareZChange}
      />
      <hr style={dividerStyle} />
      <ChipStrip
        label="n"
        values={nValues}
        selected={value.n}
        onPick={(n) => emit(n, value.l, value.m)}
        idPrefix="ctrl-n"
        labelTooltip={LABEL_TOOLTIPS.n}
        chipTooltip={(v) => N_TOOLTIPS[v]}
      />
      <ChipStrip
        label="l"
        values={lValues}
        selected={value.l}
        onPick={(l) => emit(value.n, l, value.m)}
        idPrefix="ctrl-l"
        labelTooltip={LABEL_TOOLTIPS.l}
        chipTooltip={(v) => L_TOOLTIPS[v]}
      />
      <ChipStrip
        label="m"
        values={mValues}
        selected={value.m}
        onPick={(m) => emit(value.n, value.l, m)}
        idPrefix="ctrl-m"
        labelTooltip={LABEL_TOOLTIPS.m}
        chipTooltip={mTooltip}
      />
      <hr style={dividerStyle} />
      <ColormapPicker value={colormap} onPick={onColormapChange} />
      <hr style={dividerStyle} />
      <PresetStrip
        value={value}
        onPick={(p) => emit(p.n, p.l, p.m)}
      />
      <hr style={dividerStyle} />
      <ToggleRow
        label="auto-rotate"
        labelTooltip={LABEL_TOOLTIPS.autoRotate}
        value={autoRotate}
        onChange={onAutoRotateChange}
      />
    </div>
  );
}
