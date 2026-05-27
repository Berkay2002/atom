'use client';

import { useCallback, useEffect, useRef, useState, type PointerEvent, type WheelEvent } from 'react';

import AtomCanvas from '@/components/AtomCanvas';
import Controls, { type OrbitalParams } from '@/components/Controls';
import type { ColormapName } from '@/lib/colormaps';
import {
  DEFAULT_STORED_STATE,
  loadStoredState,
  saveStoredState,
  type StoredState,
} from '@/lib/storage';

// `(n, l, m)` and `colormap` defaults match the desktop's `UiState::default()`
// — a 3d_(xy)-ish lobe with the fiery INFERNO palette. `autoRotate` and
// `hudVisible` mirror the desktop too (off / visible).
const DEFAULT_PARAMS: OrbitalParams = {
  n: DEFAULT_STORED_STATE.n,
  l: DEFAULT_STORED_STATE.l,
  m: DEFAULT_STORED_STATE.m,
};
const DEFAULT_COLORMAP: ColormapName = DEFAULT_STORED_STATE.colormap;
const DEFAULT_ELEMENT_Z = DEFAULT_STORED_STATE.elementZ;

const STORAGE_DEBOUNCE_MS = 300;

export default function Home() {
  // SSR-safe init: render defaults on the server so the client's first
  // render matches (avoids hydration warnings), then rehydrate from
  // localStorage inside a one-shot effect. The cost is a single frame of
  // default state on first paint — acceptable; the bake debounce hides
  // it anyway.
  const [elementZ, setElementZ] = useState<number>(DEFAULT_ELEMENT_Z);
  const [params, setParams] = useState<OrbitalParams>(DEFAULT_PARAMS);
  const [colormap, setColormap] = useState<ColormapName>(DEFAULT_COLORMAP);
  const [autoRotate, setAutoRotate] = useState<boolean>(DEFAULT_STORED_STATE.autoRotate);
  const [hudVisible, setHudVisible] = useState<boolean>(DEFAULT_STORED_STATE.hudVisible);
  // Gate localStorage *writes* on the rehydrate completing — otherwise
  // the first persistence effect would clobber stored state with the
  // SSR defaults before we ever read what's there.
  const hydratedRef = useRef(false);

  useEffect(() => {
    // Rehydrate from localStorage on the client. The set-in-effect rule
    // exists to discourage layout thrashing, but here it's the canonical
    // Next.js pattern for SSR-safe client-only state — we *must* render
    // defaults first to match the server HTML, then catch up.
    const stored = loadStoredState();
    /* eslint-disable react-hooks/set-state-in-effect */
    setElementZ(stored.elementZ);
    setParams({ n: stored.n, l: stored.l, m: stored.m });
    setColormap(stored.colormap);
    setAutoRotate(stored.autoRotate);
    setHudVisible(stored.hudVisible);
    /* eslint-enable react-hooks/set-state-in-effect */
    hydratedRef.current = true;
  }, []);

  // Debounced write: any persisted field changes → schedule a single
  // serialize-and-write after 300ms of quiet. Cleanup cancels the
  // pending timer, so a rapid burst (e.g. dragging through n values via
  // the keyboard / clicks) collapses to one write.
  useEffect(() => {
    if (!hydratedRef.current) return;
    const snapshot: StoredState = {
      elementZ,
      n: params.n,
      l: params.l,
      m: params.m,
      colormap,
      autoRotate,
      hudVisible,
    };
    const timer = setTimeout(() => saveStoredState(snapshot), STORAGE_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [elementZ, params.n, params.l, params.m, colormap, autoRotate, hudVisible]);

  // Global `H` shortcut toggles the HUD. Uses the functional setState
  // form so the listener stays correct even though the effect runs once
  // (no `hudVisible` in deps → no listener churn, no stale closure).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // Ignore the shortcut while the user is typing somewhere — there
      // are no text inputs in the demo today, but cheap insurance.
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || target?.isContentEditable) return;
      if (e.key === 'h' || e.key === 'H') {
        setHudVisible((v) => !v);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  const toggleHud = useCallback(() => setHudVisible((v) => !v), []);

  return (
    <>
      <AtomCanvas elementZ={elementZ} params={params} colormap={colormap} autoRotate={autoRotate} />
      {hudVisible && (
        <Controls
          elementZ={elementZ}
          onElementChange={setElementZ}
          value={params}
          onChange={setParams}
          colormap={colormap}
          onColormapChange={setColormap}
          autoRotate={autoRotate}
          onAutoRotateChange={setAutoRotate}
        />
      )}
      <EyeToggle visible={hudVisible} onToggle={toggleHud} />
    </>
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
