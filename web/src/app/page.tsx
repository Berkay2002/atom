'use client';

import { Suspense, type PointerEvent, type WheelEvent } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';

import AtomCanvas from '@/components/AtomCanvas';
import Controls from '@/components/Controls';
import TourBar from '@/components/TourBar';
import { useSceneSession } from '@/hooks/useSceneSession';

export default function Home() {
  // `useSearchParams` triggers Suspense — wrap so the rest of the page
  // can still prerender. The actual app lives in `HomeContent`.
  return (
    <Suspense fallback={null}>
      <HomeContent />
    </Suspense>
  );
}

function HomeContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const initialSearch = searchParams.toString();

  const {
    elementZ,
    params,
    colormap,
    autoRotate,
    hudVisible,
    useBareZ,
    decodeError,
    dismissDecodeError,
    tourMode,
    currentStep,
    handlePickTour,
    handleTourPrev,
    handleTourNext,
    handleTourExit,
    toggleHud,
    handleElementChange,
    handleOrbitalChange,
    handleColormapChange,
    handleAutoRotateChange,
    handleUseBareZChange,
  } = useSceneSession({
    router,
    initialSearch,
  });

  return (
    <>
      <AtomCanvas
        elementZ={elementZ}
        params={params}
        colormap={colormap}
        autoRotate={autoRotate}
        useBareZ={useBareZ}
      />
      {hudVisible && !tourMode && (
        <Controls
          elementZ={elementZ}
          onElementChange={handleElementChange}
          value={params}
          onChange={handleOrbitalChange}
          colormap={colormap}
          onColormapChange={handleColormapChange}
          autoRotate={autoRotate}
          onAutoRotateChange={handleAutoRotateChange}
          useBareZ={useBareZ}
          onUseBareZChange={handleUseBareZChange}
          onPickTour={handlePickTour}
        />
      )}
      {tourMode && currentStep && (
        <TourBar
          tourName={tourMode.tour.name}
          caption={currentStep.caption}
          stepIndex={tourMode.stepIndex}
          totalSteps={tourMode.tour.steps.length}
          onPrev={handleTourPrev}
          onNext={handleTourNext}
          onExit={handleTourExit}
        />
      )}
      <EyeToggle visible={hudVisible} onToggle={toggleHud} />
      {decodeError && <DecodeErrorBanner message={decodeError} onDismiss={dismissDecodeError} />}
    </>
  );
}

// Inline dismissable banner shown when a shared-URL `?s=...` value fails
// to decode. Deliberately tiny — no toast library, no animation tower:
// fixed-position at the bottom edge, single dismiss button, accent
// border matching the rest of the HUD chrome.
function DecodeErrorBanner({
  message,
  onDismiss,
}: {
  message: string;
  onDismiss: () => void;
}) {
  return (
    <div
      role="status"
      aria-live="polite"
      style={{
        position: 'fixed',
        bottom: 16,
        left: '50%',
        transform: 'translateX(-50%)',
        zIndex: 15,
        maxWidth: 480,
        padding: '10px 14px',
        borderRadius: 8,
        background: 'rgba(14, 12, 18, 0.9)',
        color: 'rgba(255, 255, 255, 0.85)',
        font: '12px / 1.4 system-ui, -apple-system, "Segoe UI", sans-serif',
        borderStyle: 'solid',
        borderWidth: 1,
        borderColor: 'rgba(255, 138, 76, 0.6)',
        backdropFilter: 'blur(12px) saturate(140%)',
        WebkitBackdropFilter: 'blur(12px) saturate(140%)',
        boxShadow: '0 8px 24px rgba(0, 0, 0, 0.35)',
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        pointerEvents: 'auto',
      }}
    >
      <span style={{ flex: 1 }}>{message}</span>
      <button
        type="button"
        aria-label="Dismiss"
        onClick={onDismiss}
        style={{
          background: 'transparent',
          border: 'none',
          color: 'rgba(255, 255, 255, 0.6)',
          cursor: 'pointer',
          font: 'inherit',
          padding: '2px 6px',
          borderRadius: 4,
        }}
      >
        ×
      </button>
    </div>
  );
}

// Inline icon + button. Lives outside the controls card so toggling
// `hudVisible` never hides the means of bringing the card back. Anchored
// top-right to mirror the desktop (`crates/atom-desktop/src/ui_widgets.rs`
// :: `eye_toggle` uses `Align2::RIGHT_TOP`). Dimmed at 35% opacity when
// the card is hidden — same ratio as the desktop's `ui.set_opacity(0.35)`.
function EyeToggle({ visible, onToggle }: { visible: boolean; onToggle: () => void }) {
  const stop = (e: PointerEvent | WheelEvent) => e.stopPropagation();
  return (
    <button
      type="button"
      aria-label={visible ? 'Hide controls (H)' : 'Show controls (H)'}
      aria-pressed={!visible}
      title={visible ? 'Hide controls (H)' : 'Show controls (H)'}
      onClick={onToggle}
      onPointerDown={stop}
      onPointerMove={stop}
      onPointerUp={stop}
      onWheel={stop}
      style={{
        position: 'fixed',
        top: 16,
        right: 16,
        zIndex: 11,
        width: 32,
        height: 32,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 0,
        borderRadius: 8,
        background: 'rgba(14, 12, 18, 0.85)',
        borderStyle: 'solid',
        borderWidth: 1,
        borderColor: 'rgba(255, 255, 255, 0.06)',
        color: 'rgba(255, 255, 255, 0.85)',
        opacity: visible ? 1 : 0.35,
        cursor: 'pointer',
        backdropFilter: 'blur(12px) saturate(140%)',
        WebkitBackdropFilter: 'blur(12px) saturate(140%)',
        boxShadow: '0 8px 24px rgba(0, 0, 0, 0.35)',
        transition: 'opacity 140ms ease, background 120ms ease',
      }}
    >
      {visible ? <EyeOpenIcon /> : <EyeClosedIcon />}
    </button>
  );
}

// 20×20 viewBox, 1.5px stroke — Heroicons-style outline. Inlined so we
// don't pull in an icon library for two SVGs.
function EyeOpenIcon() {
  return (
    <svg
      width="18"
      height="18"
      viewBox="0 0 20 20"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M1.5 10S4.5 4 10 4s8.5 6 8.5 6-3 6-8.5 6-8.5-6-8.5-6Z" />
      <circle cx="10" cy="10" r="2.5" />
    </svg>
  );
}

function EyeClosedIcon() {
  return (
    <svg
      width="18"
      height="18"
      viewBox="0 0 20 20"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M3 4l14 12" />
      <path d="M6.5 5.5C4 7 1.5 10 1.5 10s3 6 8.5 6c1.5 0 2.85-.4 4-1" />
      <path d="M13.5 14.5C16 13 18.5 10 18.5 10s-3-6-8.5-6c-1.1 0-2.1.22-3 .57" />
      <path d="M8 8.5a2.5 2.5 0 0 0 3.5 3.5" />
    </svg>
  );
}
