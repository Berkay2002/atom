'use client';

// In-tour overlay: caption + Prev/Next/Exit + step indicator (issue 07).
//
// Positioned as a fixed bar pinned to the bottom edge of the canvas so it
// stays out of the way of the controls card (top-left) and the HUD eye
// toggle (top-right). The bar reserves a constant minimum height so the
// caption text wrapping doesn't make the layout jump between steps.
//
// All navigation logic lives in the parent (`page.tsx`) — this component
// is purely presentational, taking the current step, totals, and three
// callbacks for prev/next/exit.

import type { CSSProperties, PointerEvent, WheelEvent } from 'react';

const TOKEN = {
  cardBg: 'rgba(14, 12, 18, 0.92)',
  accent: '#ff8a4c',
  accentDim: 'rgba(255, 138, 76, 0.30)',
  textPrimary: 'rgba(255, 255, 255, 0.92)',
  textTertiary: 'rgba(255, 255, 255, 0.45)',
  surfaceMute: 'rgba(255, 255, 255, 0.06)',
  surfaceMuteHover: 'rgba(255, 255, 255, 0.12)',
  border: 'rgba(255, 255, 255, 0.08)',
  bodySize: 13,
  labelSize: 10,
} as const;

const barStyle: CSSProperties = {
  position: 'fixed',
  left: '50%',
  bottom: 16,
  transform: 'translateX(-50%)',
  zIndex: 14,
  width: 'min(720px, calc(100vw - 32px))',
  display: 'flex',
  flexDirection: 'column',
  gap: 10,
  padding: '14px 16px',
  borderRadius: 10,
  background: TOKEN.cardBg,
  borderStyle: 'solid',
  borderWidth: 1,
  borderColor: TOKEN.border,
  color: TOKEN.textPrimary,
  font: `${TOKEN.bodySize}px / 1.45 system-ui, -apple-system, "Segoe UI", sans-serif`,
  backdropFilter: 'blur(12px) saturate(140%)',
  WebkitBackdropFilter: 'blur(12px) saturate(140%)',
  boxShadow: '0 8px 24px rgba(0, 0, 0, 0.4)',
  pointerEvents: 'auto',
  userSelect: 'none',
};

const headerRowStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: 8,
};

const tourTitleStyle: CSSProperties = {
  fontSize: TOKEN.labelSize,
  textTransform: 'uppercase',
  letterSpacing: '0.18em',
  color: TOKEN.accent,
  fontWeight: 600,
  flex: 1,
  minWidth: 0,
  // Ellipsize very long tour names instead of wrapping into the step counter.
  overflow: 'hidden',
  whiteSpace: 'nowrap',
  textOverflow: 'ellipsis',
};

const stepIndicatorStyle: CSSProperties = {
  fontSize: TOKEN.labelSize,
  textTransform: 'uppercase',
  letterSpacing: '0.16em',
  color: TOKEN.textTertiary,
  fontWeight: 500,
  fontVariantNumeric: 'tabular-nums',
  flexShrink: 0,
};

const captionStyle: CSSProperties = {
  margin: 0,
  fontSize: TOKEN.bodySize,
  lineHeight: 1.5,
  // Reserve ~3 lines of vertical space so single-line vs three-line captions
  // don't visibly resize the bar between steps. The browser still wraps
  // longer text naturally.
  minHeight: '3em',
};

const controlsRowStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: 8,
};

const navButtonsStyle: CSSProperties = {
  display: 'flex',
  gap: 6,
};

const buttonBase: CSSProperties = {
  padding: '6px 14px',
  borderRadius: 6,
  font: 'inherit',
  fontSize: 12,
  fontWeight: 500,
  cursor: 'pointer',
  borderStyle: 'solid',
  borderWidth: 1,
  borderColor: 'transparent',
  background: TOKEN.surfaceMute,
  color: TOKEN.textPrimary,
  transition: 'background 120ms ease, border-color 120ms ease, opacity 120ms ease',
};

const buttonDisabled: CSSProperties = {
  ...buttonBase,
  opacity: 0.35,
  cursor: 'not-allowed',
};

const exitButton: CSSProperties = {
  ...buttonBase,
  background: 'transparent',
  borderColor: TOKEN.border,
  color: TOKEN.textTertiary,
};

export type TourBarProps = {
  tourName: string;
  caption: string;
  /** Zero-indexed current step. */
  stepIndex: number;
  totalSteps: number;
  onPrev: () => void;
  onNext: () => void;
  onExit: () => void;
};

export default function TourBar({
  tourName,
  caption,
  stepIndex,
  totalSteps,
  onPrev,
  onNext,
  onExit,
}: TourBarProps) {
  const canPrev = stepIndex > 0;
  const canNext = stepIndex < totalSteps - 1;

  // The bar sits over the canvas — same drag-eater pattern the controls
  // card uses so a click on Prev doesn't also fire a camera-pan gesture.
  const stop = (e: PointerEvent | WheelEvent) => e.stopPropagation();

  return (
    <div
      role="region"
      aria-label={`Tour: ${tourName}`}
      style={barStyle}
      onPointerDown={stop}
      onPointerMove={stop}
      onPointerUp={stop}
      onWheel={stop}
    >
      <div style={headerRowStyle}>
        <span style={tourTitleStyle} title={tourName}>
          {tourName}
        </span>
        <span style={stepIndicatorStyle} aria-live="polite">
          Step {stepIndex + 1} of {totalSteps}
        </span>
      </div>
      <p style={captionStyle} aria-live="polite">
        {caption}
      </p>
      <div style={controlsRowStyle}>
        <button type="button" onClick={onExit} style={exitButton}>
          Exit tour
        </button>
        <div style={navButtonsStyle}>
          <button
            type="button"
            onClick={onPrev}
            disabled={!canPrev}
            aria-label="Previous step"
            style={canPrev ? buttonBase : buttonDisabled}
          >
            ← Prev
          </button>
          <button
            type="button"
            onClick={onNext}
            disabled={!canNext}
            aria-label="Next step"
            style={canNext ? buttonBase : buttonDisabled}
          >
            Next →
          </button>
        </div>
      </div>
    </div>
  );
}
