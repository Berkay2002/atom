// URL state codec — single source of truth for the `?s=...` query param.
//
// All the actual encoding / decoding lives in `atom-core::scene` (Rust)
// and is exposed via WASM as `scene_encode` / `scene_decode`. This module
// is a thin TypeScript adapter:
//
//   * `loadCodec()` — lazy-initialises the WASM module on the main thread
//     (the bake worker initialises its own copy in parallel; the cost is a
//     ~30 KB shared download cached by the browser).
//   * `encodeScene()` / `decodeScene()` — sync wrappers once the codec is
//     ready. Errors decode into `DecodeError` objects with the friendly
//     message produced by the Rust `Display` impl.
//   * `SCENE_PARAM_NAME` — the search-param key (`s`). Centralised here so
//     hydration and persistence code never typo it.
//
// The page-level glue (debounced `router.replace`, decode-failure banner,
// initial-load hydration) lives in `app/page.tsx` and reads / writes
// through these helpers — no other module should touch the URL directly.

import init, {
  scene_decode,
  scene_encode,
  type DecodedScene,
} from '../../wasm/atom_core.js';

import type { ColormapName } from '@/lib/colormaps';
import { COLORMAP_ORDER } from '@/lib/colormaps';

/** Search-param key. URL shape: `?s=v1:1/1/0/0/eff/0/1.00`. */
export const SCENE_PARAM_NAME = 's';

/** The slice-1 fields that round-trip through a URL. Mirrors the wire
 *  format in atom-core::scene; see that file for the canonical doc. */
export type SceneUrlState = {
  /** Atomic number, 1..=18. */
  elementZ: number;
  n: number;
  l: number;
  m: number;
  useBareZ: boolean;
  colormap: ColormapName;
  /** Linear exposure scalar; encoded as 2-decimal float. */
  exposure: number;
};

/** Decode-time failure — typed so the UI can show the message verbatim. */
export class SceneDecodeError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'SceneDecodeError';
  }
}

// One-shot WASM bootstrap. The bake worker loads its own instance in
// parallel — `init()` is idempotent inside a single realm, so the main
// thread paying the cost separately is fine.
let codecReady: Promise<void> | null = null;

/**
 * Initialise the WASM codec. Idempotent — repeated calls return the same
 * pending promise. Resolves once `scene_encode` / `scene_decode` are
 * callable.
 */
export function loadCodec(): Promise<void> {
  if (!codecReady) {
    codecReady = init().then(() => undefined);
  }
  return codecReady;
}

/**
 * Encode a `SceneUrlState` to the `v1:` URL string by calling into the
 * shared Rust implementation. `loadCodec()` must have resolved before
 * calling. Throws if WASM isn't initialised — this is a programmer
 * error, not a runtime failure.
 */
export function encodeScene(state: SceneUrlState): string {
  // `colormap_id` is the index into the canonical order; we store the
  // string name in React state but flatten to a numeric id on the wire so
  // adding a colormap in the future is a no-op for old URLs.
  const colormapId = colormapNameToId(state.colormap);
  return scene_encode(
    state.elementZ,
    state.n,
    state.l,
    state.m,
    state.useBareZ,
    colormapId,
    state.exposure,
  );
}

/**
 * Decode a `v1:` URL string. Throws `SceneDecodeError` with the Rust-side
 * `Display` message on any malformed input — the page-level glue catches
 * this and shows a non-blocking banner.
 */
export function decodeScene(s: string): SceneUrlState {
  let decoded: DecodedScene;
  try {
    decoded = scene_decode(s);
  } catch (err) {
    // wasm-bindgen turns Rust's `JsError` into a regular `Error` whose
    // message is the `DecodeError::Display` output we picked in
    // `atom-core::scene`. Re-wrap so callers can `instanceof` check.
    const message = err instanceof Error ? err.message : String(err);
    throw new SceneDecodeError(message);
  }
  const result: SceneUrlState = {
    elementZ: decoded.element_z,
    n: decoded.n,
    l: decoded.l,
    m: decoded.m,
    useBareZ: decoded.use_bare_z,
    colormap: colormapIdToName(decoded.colormap_id),
    exposure: decoded.exposure,
  };
  // The `DecodedScene` holds a pointer into wasm memory; release it now
  // that we've copied the primitives out.
  decoded.free();
  return result;
}

/**
 * Extract the encoded scene string from `location.search`, if any. Returns
 * `null` on the server (no `window`) or when the param is absent.
 */
export function readSceneParam(searchString: string | null | undefined): string | null {
  if (!searchString) return null;
  // Accept both `?foo=bar` and `foo=bar` to keep callers (page effect vs.
  // `useSearchParams.toString()`) ergonomic.
  const stripped = searchString.startsWith('?') ? searchString.slice(1) : searchString;
  const params = new URLSearchParams(stripped);
  const raw = params.get(SCENE_PARAM_NAME);
  return raw && raw.length > 0 ? raw : null;
}

/**
 * Convenience: build a new search string with `?s=<encoded>` set, preserving
 * any other existing params. Used by the page's debounced URL writer.
 */
export function withSceneParam(currentSearch: string | null | undefined, encoded: string): string {
  const stripped = currentSearch?.startsWith('?')
    ? currentSearch.slice(1)
    : currentSearch ?? '';
  const params = new URLSearchParams(stripped);
  params.set(SCENE_PARAM_NAME, encoded);
  return params.toString();
}

function colormapNameToId(name: ColormapName): number {
  // Canonical mapping = position in COLORMAP_ORDER. INFERNO=0, …, ELECTRON_BLUE=5.
  const id = COLORMAP_ORDER.indexOf(name);
  // `name: ColormapName` makes this provably unreachable, but a fallback
  // to INFERNO keeps a future enum addition that's missing from the order
  // table from crashing the URL writer.
  return id < 0 ? 0 : id;
}

function colormapIdToName(id: number): ColormapName {
  // Out-of-range colormap ids decode to the first colormap rather than
  // throwing — old URLs with a higher id (e.g. someone shares a v1 URL
  // from a fork that adds a 7th colormap) still render *something*.
  if (id < 0 || id >= COLORMAP_ORDER.length) return COLORMAP_ORDER[0];
  return COLORMAP_ORDER[id];
}
