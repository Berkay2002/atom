'use client';

import { Suspense, useCallback, useEffect, useRef, useState, type PointerEvent, type WheelEvent } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';

import AtomCanvas from '@/components/AtomCanvas';
import Controls, { type OrbitalParams } from '@/components/Controls';
import TourBar from '@/components/TourBar';
import type { ColormapName } from '@/lib/colormaps';
import { homoFor } from '@/lib/element-homo';
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
import {
  clearTourParams,
  loadTour,
  readTourParams,
  TOUR_SLUG_PARAM,
  TOUR_STEP_PARAM,
  withTourParams,
  type Tour,
} from '@/lib/tours';

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

// In tour mode, the page is driven by `(tour, stepIndex)` instead of the
// individual scene fields. We still keep the scene state in sync because
// the canvas reads from those — the tour just *writes* to them on each
// Prev/Next.
type TourMode = {
  tour: Tour;
  stepIndex: number;
};

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
  // Bare-Z toggle — surfaced in the Controls UI (issue 03). Persisted to
  // localStorage and to the shareable URL so the user's choice survives
  // reloads either way.
  const [useBareZ, setUseBareZ] = useState<boolean>(DEFAULT_STORED_STATE.useBareZ);
  // Decode-error banner. Cleared on dismiss; only ever set once per page
  // load (URL hydration is one-shot).
  const [decodeError, setDecodeError] = useState<string | null>(null);
  // Active guided tour, or null in free-sandbox mode (issue 07). When set,
  // the Controls panel is hidden in favour of the TourBar overlay and the
  // `?s=` URL writer is paused — the tour params own the URL instead.
  const [tourMode, setTourMode] = useState<TourMode | null>(null);
  // Gate persistence writes on the rehydrate completing — otherwise the
  // first effect would clobber stored state / the URL with SSR defaults.
  const hydratedRef = useRef(false);

  const router = useRouter();
  const searchParams = useSearchParams();

  // Apply a tour step's scene string to the page-level scene state. Pure
  // (no UI side effects), wrapped in useCallback so the effects that call
  // it stay stable.
  const applySceneString = useCallback((sceneStr: string): boolean => {
    try {
      const decoded = decodeScene(sceneStr);
      setElementZ(decoded.elementZ);
      setParams({ n: decoded.n, l: decoded.l, m: decoded.m });
      setColormap(decoded.colormap);
      setUseBareZ(decoded.useBareZ);
      return true;
    } catch (err) {
      const message = err instanceof SceneDecodeError ? err.message : String(err);
      setDecodeError(`Tour step failed to decode: ${message}`);
      return false;
    }
  }, []);

  useEffect(() => {
    // One-shot hydrate. Precedence: tour params → URL scene → localStorage
    // → defaults. The tour path is async (fetch + parse) so we still
    // paint localStorage/defaults synchronously underneath so the canvas
    // isn't blank while the tour JSON downloads.
    let cancelled = false;

    const search = searchParams.toString();
    const tourParams = readTourParams(search);
    const urlScene = tourParams ? null : readSceneParam(search);
    const stored = loadStoredState();

    // Always paint stored state first so the canvas has something to show.
    /* eslint-disable react-hooks/set-state-in-effect */
    setElementZ(stored.elementZ);
    setParams({ n: stored.n, l: stored.l, m: stored.m });
    setColormap(stored.colormap);
    setAutoRotate(stored.autoRotate);
    setHudVisible(stored.hudVisible);
    setUseBareZ(stored.useBareZ);
    /* eslint-enable react-hooks/set-state-in-effect */

    if (tourParams) {
      // Deep-link into a tour: fetch the JSON, then enter tour mode at
      // the requested step (clamped to a valid index). WASM has to be
      // ready before we can decode the step's scene string.
      Promise.all([loadCodec(), loadTour(tourParams.slug)])
        .then(([, tour]) => {
          if (cancelled) return;
          const clampedStep = Math.min(Math.max(0, tourParams.step), tour.steps.length - 1);
          setTourMode({ tour, stepIndex: clampedStep });
          applySceneString(tour.steps[clampedStep].scene);
        })
        .catch((err: unknown) => {
          if (cancelled) return;
          const message = err instanceof Error ? err.message : String(err);
          setDecodeError(`Couldn't load tour: ${message}.`);
        })
        .finally(() => {
          if (!cancelled) hydratedRef.current = true;
        });
    } else if (urlScene) {
      // Hydrate from `?s=...` once WASM is up.
      loadCodec().then(() => {
        if (cancelled) return;
        try {
          const decoded = decodeScene(urlScene);
          setElementZ(decoded.elementZ);
          setParams({ n: decoded.n, l: decoded.l, m: decoded.m });
          setColormap(decoded.colormap);
          setUseBareZ(decoded.useBareZ);
        } catch (err) {
          const message = err instanceof SceneDecodeError ? err.message : String(err);
          setDecodeError(`Couldn't load shared view: ${message}. Using defaults.`);
        } finally {
          hydratedRef.current = true;
        }
      });
    } else {
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
  // writer below. Skipped during tour mode so mid-tour states don't
  // overwrite the user's pre-tour saved selection.
  useEffect(() => {
    if (!hydratedRef.current) return;
    if (tourMode) return;
    const snapshot: StoredState = {
      elementZ,
      n: params.n,
      l: params.l,
      m: params.m,
      colormap,
      autoRotate,
      hudVisible,
      useBareZ,
    };
    const timer = setTimeout(() => saveStoredState(snapshot), STORAGE_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [elementZ, params.n, params.l, params.m, colormap, autoRotate, hudVisible, useBareZ, tourMode]);

  // Debounced URL writer — encode the current scene state and push it
  // into the address bar via `router.replace` so the back button stays
  // useful. Skipped until hydration completes so we never paint the SSR
  // defaults into the URL.
  //
  // While a tour is active the tour params (`?tour=&step=`) are
  // authoritative — we suppress the `?s=` writer so the two URL formats
  // don't fight over the address bar on every Prev/Next.
  useEffect(() => {
    if (!hydratedRef.current) return;
    if (tourMode) return;
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
  }, [elementZ, params.n, params.l, params.m, colormap, useBareZ, router, tourMode]);

  // Tour-mode URL writer — keeps `?tour=&step=` in the address bar
  // whenever the active step changes. Cheaper than the scene writer (no
  // WASM encoding), so it runs synchronously on the navigation tick.
  useEffect(() => {
    if (!hydratedRef.current) return;
    if (!tourMode) return;
    const current = new URLSearchParams(window.location.search);
    const next = withTourParams(current.toString(), tourMode.tour.slug, tourMode.stepIndex);
    const nextStr = next.toString();
    if (current.toString() === nextStr) return; // no-op
    router.replace(`${window.location.pathname}?${nextStr}`, { scroll: false });
  }, [tourMode, router]);

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

  // Element-picker handler: snap (n, l, m) to the picked element's HOMO
  // so the canvas always lands on that element's iconic orbital. Without
  // the snap, leaving e.g. a 3d_xz preset selected and switching to
  // Carbon would silently clamp z_eff to 0 (Slater shielding for an
  // unoccupied d-orbital with too many same-shell screeners) and the
  // cloud would disappear. The user can still navigate to any (n, l, m)
  // afterwards via the chip strips or preset chips.
  const handleElementChange = useCallback((nextZ: number) => {
    setElementZ(nextZ);
    const homo = homoFor(nextZ);
    setParams({ n: homo.n, l: homo.l, m: homo.m });
  }, []);

  // --- Tour mode handlers ---------------------------------------------------

  const handlePickTour = useCallback(
    (slug: string) => {
      // Fetch + enter tour mode at step 0. The Promise chain is detached
      // (we don't await in an event handler) — errors land in the decode
      // banner instead of an uncaught rejection.
      loadCodec()
        .then(() => loadTour(slug))
        .then((tour) => {
          setTourMode({ tour, stepIndex: 0 });
          applySceneString(tour.steps[0].scene);
        })
        .catch((err: unknown) => {
          const message = err instanceof Error ? err.message : String(err);
          setDecodeError(`Couldn't load tour: ${message}`);
        });
    },
    [applySceneString],
  );

  const handleTourPrev = useCallback(() => {
    setTourMode((current) => {
      if (!current) return current;
      if (current.stepIndex <= 0) return current;
      const nextIdx = current.stepIndex - 1;
      applySceneString(current.tour.steps[nextIdx].scene);
      return { ...current, stepIndex: nextIdx };
    });
  }, [applySceneString]);

  const handleTourNext = useCallback(() => {
    setTourMode((current) => {
      if (!current) return current;
      if (current.stepIndex >= current.tour.steps.length - 1) return current;
      const nextIdx = current.stepIndex + 1;
      applySceneString(current.tour.steps[nextIdx].scene);
      return { ...current, stepIndex: nextIdx };
    });
  }, [applySceneString]);

  const handleTourExit = useCallback(() => {
    // Drop tour params and let the standard `?s=` URL writer take over on
    // the next tick. We strip the tour params synchronously here so the
    // address bar update can't lag behind state by one frame.
    setTourMode(null);
    const cleared = clearTourParams(window.location.search);
    // We *don't* set `?s=` here — the regular URL writer effect picks up
    // immediately (tourMode just flipped false) and writes the encoded
    // scene on its normal debounce.
    cleared.delete(TOUR_SLUG_PARAM);
    cleared.delete(TOUR_STEP_PARAM);
    const query = cleared.toString();
    router.replace(`${window.location.pathname}${query ? `?${query}` : ''}`, {
      scroll: false,
    });
  }, [router]);

  const currentStep = tourMode ? tourMode.tour.steps[tourMode.stepIndex] : null;

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
          onChange={setParams}
          colormap={colormap}
          onColormapChange={setColormap}
          autoRotate={autoRotate}
          onAutoRotateChange={setAutoRotate}
          useBareZ={useBareZ}
          onUseBareZChange={setUseBareZ}
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
