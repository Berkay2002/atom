// Plain-language caption for the currently-rendered scene (issue 06).
//
// The string itself is composed by `atom-core::caption(&Scene)` (Rust)
// and surfaced through the `scene_caption` WASM export. This module is a
// tiny TS adapter: lazy-init the WASM module, then call the export.
//
// React surface is `useSceneCaption(...)` — a hook that mirrors the
// scene parameters into a string and tracks the WASM-ready state so the
// initial paint doesn't flash an empty caption. The hook returns `null`
// until the codec is loaded, which callers render as a 1-line skeleton
// (or just hide the slot) rather than committing to a placeholder
// string that drifts from the Rust output.
//
// Why a dedicated module instead of folding into `scene-url.ts`?
//   * The URL codec is loaded eagerly on page hydrate; the caption can
//     be fetched lazily after the first paint and is irrelevant on the
//     server. Splitting keeps the URL hydration path slim.
//   * Tests for the URL codec don't touch WASM (see `scene-url.test.ts`
//     — Rust covers the codec exhaustively); this module's tests would
//     have to import WASM, which is jsdom-hostile. Keeping the two
//     concerns separate means we don't introduce a WASM import cost into
//     unrelated tests.

import { useEffect, useState } from 'react';

import init, { scene_caption } from '../../wasm/atom_core.js';

/** Single-atom scene parameters the caption depends on. Mirrors the
 *  shape `scene_caption` expects across the FFI boundary. */
export type SceneCaptionParams = {
  elementZ: number;
  n: number;
  l: number;
  m: number;
};

// One-shot WASM bootstrap, intentionally separate from the URL codec's
// `loadCodec` so this module can stand on its own (and so a future page
// that only needs captions doesn't drag in the URL writer).
let captionReady: Promise<void> | null = null;

function loadCaption(): Promise<void> {
  if (!captionReady) {
    captionReady = init().then(() => undefined);
  }
  return captionReady;
}

/**
 * Synchronous wrapper around the `scene_caption` WASM export. The codec
 * must already be loaded; callers should go through `useSceneCaption`
 * which handles the load on their behalf.
 */
export function sceneCaption(p: SceneCaptionParams): string {
  return scene_caption(p.elementZ, p.n, p.l, p.m);
}

/**
 * React hook that returns the caption string for a single-atom scene.
 * Returns `null` until the WASM module has loaded — callers render
 * a skeleton (or hide the slot) during that brief window.
 *
 * Re-runs whenever any of (elementZ, n, l, m) changes, which is the
 * full set of inputs the slice-1 caption depends on.
 */
export function useSceneCaption(p: SceneCaptionParams): string | null {
  const { elementZ, n, l, m } = p;
  // `ready` toggles once after the WASM init resolves. We avoid storing
  // the caption string in state and re-deriving on each render instead:
  // the WASM call is cheap (a couple hundred ns) and skipping the
  // useState shimmy keeps the hook from triggering an extra commit on
  // each param change.
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void loadCaption().then(() => {
      if (!cancelled) setReady(true);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  if (!ready) return null;
  return sceneCaption({ elementZ, n, l, m });
}
