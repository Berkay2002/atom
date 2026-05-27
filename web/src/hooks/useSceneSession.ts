'use client';

import { useCallback, useEffect, useRef, useState } from 'react';

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

export type SceneOrbitalParams = {
  n: number;
  l: number;
  m: number;
};

type SceneSessionRouter = {
  replace: (href: string, options?: { scroll?: boolean }) => void;
};

type UseSceneSessionArgs = {
  router: SceneSessionRouter;
  initialSearch: string;
  ignoreSceneUrl?: boolean;
  persistencePaused?: boolean;
};

export type TourMode = {
  tour: Tour;
  stepIndex: number;
};

export const DEFAULT_PARAMS: SceneOrbitalParams = {
  n: DEFAULT_STORED_STATE.n,
  l: DEFAULT_STORED_STATE.l,
  m: DEFAULT_STORED_STATE.m,
};

export const DEFAULT_COLORMAP: ColormapName = DEFAULT_STORED_STATE.colormap;
export const DEFAULT_ELEMENT_Z = DEFAULT_STORED_STATE.elementZ;

export const STORAGE_DEBOUNCE_MS = 300;
// URL writes debounce — separate from the storage write so dragging the
// camera / spamming chips doesn't burn 60 history-replacements per second.
export const URL_DEBOUNCE_MS = 250;

export function useSceneSession({
  router,
  initialSearch,
  ignoreSceneUrl = false,
  persistencePaused = false,
}: UseSceneSessionArgs) {
  const [elementZ, setElementZ] = useState<number>(DEFAULT_ELEMENT_Z);
  const [params, setParams] = useState<SceneOrbitalParams>(DEFAULT_PARAMS);
  const [colormap, setColormap] = useState<ColormapName>(DEFAULT_COLORMAP);
  const [autoRotate, setAutoRotate] = useState<boolean>(DEFAULT_STORED_STATE.autoRotate);
  const [hudVisible, setHudVisible] = useState<boolean>(DEFAULT_STORED_STATE.hudVisible);
  const [useBareZ, setUseBareZ] = useState<boolean>(DEFAULT_STORED_STATE.useBareZ);
  const [decodeError, setDecodeError] = useState<string | null>(null);
  const [tourMode, setTourMode] = useState<TourMode | null>(null);

  const hydratedRef = useRef(false);
  const initialSearchRef = useRef(initialSearch);
  const ignoreSceneUrlRef = useRef(ignoreSceneUrl);
  const initialTourParamsRef = useRef(readTourParams(initialSearch));
  const tourRequestRef = useRef(0);

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

  const applyTourStep = useCallback(
    (tour: Tour, stepIndex: number) => {
      applySceneString(tour.steps[stepIndex].scene);
      setTourMode({ tour, stepIndex });
    },
    [applySceneString],
  );

  useEffect(() => {
    let cancelled = false;

    const tourParams = initialTourParamsRef.current;
    const urlScene =
      tourParams || ignoreSceneUrlRef.current ? null : readSceneParam(initialSearchRef.current);
    const stored = loadStoredState();

    /* eslint-disable react-hooks/set-state-in-effect */
    setElementZ(stored.elementZ);
    setParams({ n: stored.n, l: stored.l, m: stored.m });
    setColormap(stored.colormap);
    setAutoRotate(stored.autoRotate);
    setHudVisible(stored.hudVisible);
    setUseBareZ(stored.useBareZ);
    /* eslint-enable react-hooks/set-state-in-effect */

    if (tourParams) {
      const requestId = ++tourRequestRef.current;
      Promise.all([loadCodec(), loadTour(tourParams.slug)])
        .then(([, tour]) => {
          if (cancelled || requestId !== tourRequestRef.current) return;
          const clampedStep = Math.min(Math.max(0, tourParams.step), tour.steps.length - 1);
          hydratedRef.current = true;
          applyTourStep(tour, clampedStep);
        })
        .catch((err: unknown) => {
          if (cancelled || requestId !== tourRequestRef.current) return;
          const message = err instanceof Error ? err.message : String(err);
          hydratedRef.current = true;
          setDecodeError(`Couldn't load tour: ${message}.`);
        });
    } else if (urlScene) {
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
      void loadCodec();
    }

    return () => {
      cancelled = true;
    };
  }, [applyTourStep]);

  useEffect(() => {
    if (!hydratedRef.current) return;
    if (persistencePaused) return;
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
  }, [
    elementZ,
    params.n,
    params.l,
    params.m,
    colormap,
    autoRotate,
    hudVisible,
    useBareZ,
    persistencePaused,
    tourMode,
  ]);

  useEffect(() => {
    if (!hydratedRef.current) return;
    if (persistencePaused) return;
    if (tourMode) return;
    const snapshot: SceneUrlState = {
      elementZ,
      n: params.n,
      l: params.l,
      m: params.m,
      useBareZ,
      colormap,
      exposure: 1.0,
    };
    const timer = setTimeout(() => {
      void loadCodec().then(() => {
        let encoded: string;
        try {
          encoded = encodeScene(snapshot);
        } catch {
          return;
        }
        const current = new URLSearchParams(window.location.search);
        if (current.get(SCENE_PARAM_NAME) === encoded) return;
        current.set(SCENE_PARAM_NAME, encoded);
        router.replace(`${window.location.pathname}?${current.toString()}`, {
          scroll: false,
        });
      });
    }, URL_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [elementZ, params.n, params.l, params.m, colormap, useBareZ, router, persistencePaused, tourMode]);

  useEffect(() => {
    if (!hydratedRef.current) return;
    if (!tourMode) return;
    const current = new URLSearchParams(window.location.search);
    const next = withTourParams(current.toString(), tourMode.tour.slug, tourMode.stepIndex);
    const nextStr = next.toString();
    if (current.toString() === nextStr) return;
    router.replace(`${window.location.pathname}?${nextStr}`, { scroll: false });
  }, [tourMode, router]);

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
  const handleOrbitalChange = useCallback((next: SceneOrbitalParams) => setParams(next), []);
  const handleColormapChange = useCallback((next: ColormapName) => setColormap(next), []);
  const handleAutoRotateChange = useCallback((next: boolean) => setAutoRotate(next), []);
  const handleUseBareZChange = useCallback((next: boolean) => setUseBareZ(next), []);
  const handleElementChange = useCallback((nextZ: number) => {
    setElementZ(nextZ);
    const homo = homoFor(nextZ);
    setParams({ n: homo.n, l: homo.l, m: homo.m });
  }, []);
  const handlePickTour = useCallback(
    (slug: string) => {
      const requestId = ++tourRequestRef.current;
      loadCodec()
        .then(() => loadTour(slug))
        .then((tour) => {
          if (requestId !== tourRequestRef.current) return;
          applyTourStep(tour, 0);
        })
        .catch((err: unknown) => {
          if (requestId !== tourRequestRef.current) return;
          const message = err instanceof Error ? err.message : String(err);
          setDecodeError(`Couldn't load tour: ${message}`);
        });
    },
    [applyTourStep],
  );
  const handleTourPrev = useCallback(() => {
    if (!tourMode) return;
    if (tourMode.stepIndex <= 0) return;
    applyTourStep(tourMode.tour, tourMode.stepIndex - 1);
  }, [applyTourStep, tourMode]);
  const handleTourNext = useCallback(() => {
    if (!tourMode) return;
    if (tourMode.stepIndex >= tourMode.tour.steps.length - 1) return;
    applyTourStep(tourMode.tour, tourMode.stepIndex + 1);
  }, [applyTourStep, tourMode]);
  const handleTourExit = useCallback(() => {
    tourRequestRef.current += 1;
    setTourMode(null);
    const cleared = clearTourParams(window.location.search);
    cleared.delete(TOUR_SLUG_PARAM);
    cleared.delete(TOUR_STEP_PARAM);
    const query = cleared.toString();
    router.replace(`${window.location.pathname}${query ? `?${query}` : ''}`, {
      scroll: false,
    });
  }, [router]);

  const currentStep = tourMode ? tourMode.tour.steps[tourMode.stepIndex] : null;

  return {
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
  };
}
