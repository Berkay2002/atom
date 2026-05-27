'use client';

// Tour-discovery UI (issue 07). A labelled "tours" section in the controls
// card listing every available tour by name. Clicking a row enters tour
// mode — the actual navigation + caption bar live in `TourBar`.
//
// The list is fetched once via `loadTourIndex()` and cached, so toggling
// the HUD on/off doesn't refetch. While loading, we render nothing (the
// label is suppressed too) so there's no flash of empty section.

import { useEffect, useState, type CSSProperties } from 'react';

import { loadTourIndex, type TourIndexEntry } from '@/lib/tours';

// Mirror the token vocabulary in `Controls.tsx` so visuals match the rest
// of the card. We duplicate rather than export from Controls because the
// alternative is a public-token-module refactor that isn't worth shipping
// for one component.
const TOKEN = {
  accent: '#ff8a4c',
  accentDim: 'rgba(255, 138, 76, 0.30)',
  textPrimary: 'rgba(255, 255, 255, 0.85)',
  textTertiary: 'rgba(255, 255, 255, 0.35)',
  surfaceMute: 'rgba(255, 255, 255, 0.05)',
  surfaceMuteHover: 'rgba(255, 255, 255, 0.09)',
  border: 'rgba(255, 255, 255, 0.06)',
  bodySize: 12,
  labelSize: 9,
} as const;

const rowStyle: CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  gap: 4,
};

const labelStyle: CSSProperties = {
  fontSize: TOKEN.labelSize,
  textTransform: 'uppercase',
  letterSpacing: '0.16em',
  color: TOKEN.textTertiary,
  fontWeight: 500,
};

const listStyle: CSSProperties = {
  display: 'flex',
  flexDirection: 'column',
  gap: 4,
};

const tourButtonBase: CSSProperties = {
  // Full-width tappable row — names can be long so a wrap-friendly layout
  // beats the pill chip used elsewhere.
  width: '100%',
  textAlign: 'left',
  padding: '6px 10px',
  borderRadius: 6,
  font: 'inherit',
  fontSize: TOKEN.bodySize,
  lineHeight: 1.3,
  cursor: 'pointer',
  borderStyle: 'solid',
  borderWidth: 1,
  borderColor: 'transparent',
  background: TOKEN.surfaceMute,
  color: TOKEN.textPrimary,
  transition: 'background 120ms ease, border-color 120ms ease',
};

const tourButtonHover: CSSProperties = {
  ...tourButtonBase,
  background: TOKEN.surfaceMuteHover,
  borderColor: TOKEN.accentDim,
};

const errorStyle: CSSProperties = {
  fontSize: 11,
  color: 'rgba(255, 138, 76, 0.85)',
  margin: 0,
};

export type TourPickerProps = {
  onPickTour: (slug: string) => void;
};

export default function TourPicker({ onPickTour }: TourPickerProps) {
  const [tours, setTours] = useState<TourIndexEntry[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [hovered, setHovered] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    loadTourIndex()
      .then((entries) => {
        if (!cancelled) setTours(entries);
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Loading state: render the label so the layout doesn't pop in once
  // tours resolve, but keep the body empty.
  if (!tours && !error) {
    return (
      <div style={rowStyle}>
        <span style={labelStyle}>tours</span>
      </div>
    );
  }

  if (error) {
    return (
      <div style={rowStyle}>
        <span style={labelStyle}>tours</span>
        <p style={errorStyle}>Tours unavailable: {error}</p>
      </div>
    );
  }

  return (
    <div style={rowStyle}>
      <span style={labelStyle}>tours</span>
      <div style={listStyle}>
        {tours!.map((t) => {
          const isHovered = hovered === t.slug;
          return (
            <button
              key={t.slug}
              type="button"
              onClick={() => onPickTour(t.slug)}
              onPointerEnter={() => setHovered(t.slug)}
              onPointerLeave={() => setHovered((h) => (h === t.slug ? null : h))}
              style={isHovered ? tourButtonHover : tourButtonBase}
              title={t.description}
            >
              {t.name}
            </button>
          );
        })}
      </div>
    </div>
  );
}
