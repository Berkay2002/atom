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

import { useState, type CSSProperties, type PointerEvent } from 'react';

export type OrbitalParams = {
  n: number;
  l: number;
  m: number;
};

export type ControlsProps = {
  value: OrbitalParams;
  onChange: (next: OrbitalParams) => void;
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

function range(lo: number, hi: number): number[] {
  const out: number[] = [];
  for (let i = lo; i <= hi; i += 1) out.push(i);
  return out;
}

// Native-title tooltips so non-physicist visitors can hover any chip or
// row label and learn what the quantum number means. Browser-native
// `title` attribute — no library, no positioning logic.
const LABEL_TOOLTIPS: Record<string, string> = {
  n: 'Principal quantum number — energy level / size of the orbital',
  l: 'Orbital angular momentum — shape of the orbital (0=s, 1=p, 2=d, 3=f)',
  m: 'Magnetic quantum number — orientation of the orbital in space (−l ≤ m ≤ +l)',
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

export default function Controls({ value, onChange }: ControlsProps) {
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
    </div>
  );
}
