'use client';

import { Suspense, useCallback, useEffect, useRef, useState, type PointerEvent, type WheelEvent } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';

import AtomCanvas from '@/components/AtomCanvas';
import Controls, { type OrbitalParams } from '@/components/Controls';
import type { ColormapName } from '@/lib/colormaps';
import {
  decodeScene,
  encodeScene,
  loadCodec,
  readSceneParam,
  SceneDecodeError,
  SCENE_PARAM_NAME,
  type SceneUrlState,
} from '@/lib/scene-url';
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
// URL writes debounce — separate from the storage write so dragging the
// camera / spamming chips doesn't burn 60 history-replacements per second.
// 250ms feels instant to a human and easily collapses a rapid burst.
const URL_DEBOUNCE_MS = 250;

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
  // SSR-safe init: render defaults on the server so the client's first
  // render matches (avoids hydration warnings), then rehydrate inside a
  // one-shot effect. URL has precedence over localStorage; both write
  // back into the same state.
  const [elementZ, setElementZ] = useState<number>(DEFAULT_ELEMENT_Z);
  const [params, setParams] = useState<OrbitalParams>(DEFAULT_PARAMS);
  const [colormap, setColormap] = useState<ColormapName>(DEFAULT_COLORMAP);
  const [autoRotate, setAutoRotate] = useState<boolean>(DEFAULT_STORED_STATE.autoRotate);
  const [hudVisible, setHudVisible] = useState<boolean>(DEFAULT_STORED_STATE.hudVisible);
  // Bare-Z toggle ships in issue 03; the field exists in state today so
  // shareable URLs round-trip the flag cleanly when that toggle lands.
  const [useBareZ, setUseBareZ] = useState<boolean>(false);
  // Decode-error banner. Cleared on dismiss; only ever set once per page
  // load (URL hydration is one-shot).
  const [decodeError, setDecodeError] = useState<string | null>(null);
  // Gate persistence writes on the rehydrate completing — otherwise the
  // first effect would clobber stored state / the URL with SSR defaults.
  const hydratedRef = useRef(false);

  const router = useRouter();
  const searchParams = useSearchParams();

  useEffect(() => {
    // One-shot hydrate. Precedence: URL → localStorage → defaults.
    // We block URL writes until WASM is ready (`loadCodec`) but allow the
    // localStorage path to run synchronously so the page paints fast even
    // when the wasm module is still in flight.
    let cancelled = false;

    const urlScene = readSceneParam(searchParams.toString());
    const stored = loadStoredState();

    if (urlScene) {
      // Hydrate from URL once WASM is up. While we wait we still paint
      // localStorage / defaults so the visualizer isn't blank.
      /* eslint-disable react-hooks/set-state-in-effect */
      setElementZ(stored.elementZ);
      setParams({ n: stored.n, l: stored.l, m: stored.m });
      setColormap(stored.colormap);
      setAutoRotate(stored.autoRotate);
      setHudVisible(stored.hudVisible);
      /* eslint-enable react-hooks/set-state-in-effect */

      loadCodec().then(() => {
        if (cancelled) return;
        try {
          const decoded = decodeScene(urlScene);
          // set-state-in-effect rule doesn't apply here — we're inside a
          // promise resolution, not directly in the effect body.
          setElementZ(decoded.elementZ);
          setParams({ n: decoded.n, l: decoded.l, m: decoded.m });
          setColormap(decoded.colormap);
          setUseBareZ(decoded.useBareZ);
        } catch (err) {
          // Malformed URL — defaults already painted, show a banner so
          // the user knows the link they followed wasn't valid.
          const message = err instanceof SceneDecodeError ? err.message : String(err);
          setDecodeError(`Couldn't load shared view: ${message}. Using defaults.`);
        } finally {
          hydratedRef.current = true;
        }
      });
    } else {
      setElementZ(stored.elementZ);
      setParams({ n: stored.n, l: stored.l, m: stored.m });
      setColormap(stored.colormap);
      setAutoRotate(stored.autoRotate);
      setHudVisible(stored.hudVisible);
      hydratedRef.current = true;
      // Kick off the codec init proactively so the *next* state change
      // can write the URL without waiting for wasm-bindgen on the
      // critical path.
      void loadCodec();
    }

    return () => {
      cancelled = true;
    };
    // searchParams intentionally not in deps — URL hydration runs once,
    // user-driven param changes after that are driven by state, not by
    // the URL.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Debounced localStorage write — same shape as before, plus the URL
  // writer below.
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

  // Debounced URL writer — encode the current scene state and push it
  // into the address bar via `router.replace` so the back button stays
  // useful. Skipped until hydration completes so we never paint the SSR
  // defaults into the URL.
  useEffect(() => {
    if (!hydratedRef.current) return;
    const snapshot: SceneUrlState = {
      elementZ,
      n: params.n,
      l: params.l,
      m: params.m,
      useBareZ,
      colormap,
      exposure: 1.0, // No UI slider yet; locked at 1× for slice 1.
    };
    const timer = setTimeout(() => {
      // Codec may not have finished initialising yet on a very fast page;
      // wait, then write. `loadCodec` resolves immediately on subsequent
      // calls so this isn't a perf concern.
      void loadCodec().then(() => {
        let encoded: string;
        try {
          encoded = encodeScene(snapshot);
        } catch {
          // encodeScene only fails on a programmer error (multi-atom) for
          // this state shape — swallow rather than crash the page.
          return;
        }
        const current = new URLSearchParams(window.location.search);
        if (current.get(SCENE_PARAM_NAME) === encoded) return; // no-op write
        current.set(SCENE_PARAM_NAME, encoded);
        router.replace(`${window.location.pathname}?${current.toString()}`, {
          scroll: false,
        });
      });
    }, URL_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [elementZ, params.n, params.l, params.m, colormap, useBareZ, router]);

  // Global `H` shortcut toggles the HUD. Uses the functional setState
  // form so the listener stays correct even though the effect runs once.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
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
  const dismissDecodeError = useCallback(() => setDecodeError(null), []);

  return (
    <>
      <AtomCanvas
        elementZ={elementZ}
        params={params}
        colormap={colormap}
        autoRotate={autoRotate}
        useBareZ={useBareZ}
      />
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
