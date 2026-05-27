// Persisted demo state shape + pure parse helpers.
//
// One JSON blob under `STORAGE_KEY` holds everything that survives a
// page reload: the current orbital (n, l, m), the colormap pick, the
// auto-rotate toggle, and the HUD-visible toggle. Reads are tolerant —
// invalid shapes fall back to defaults rather than throwing, and out-of-
// range (n, l, m) combinations are clamped so a hand-edited localStorage
// can never crash the page.
//
// Everything in this module is pure: no React, no DOM. The page does the
// reading/writing via small wrappers that guard `typeof window`.

import { COLORMAP_ORDER, type ColormapName } from '@/lib/colormaps';

export const STORAGE_KEY = 'atom-demo-state';

export type StoredState = {
  /** Atomic number of the selected element (1..=18). */
  elementZ: number;
  n: number;
  l: number;
  m: number;
  colormap: ColormapName;
  autoRotate: boolean;
  hudVisible: boolean;
  /** Bare-Z toggle. When `true`, the bake uses the bare atomic number
   *  `Z` instead of the Slater-screened `z_eff`. Persisted so the
   *  toggle survives a reload even without a URL hash. */
  useBareZ: boolean;
};

// Slice 05 defaults — kept here so the page and the parser agree on the
// canonical fallback. Both `parseStoredState` and `loadStoredState`
// return these when no valid stored blob is found.
export const DEFAULT_STORED_STATE: StoredState = {
  elementZ: 1, // Hydrogen — the bare-Z baseline everyone has seen before.
  // Hydrogen 1s — the canonical "what is an orbital" shot. The previous
  // default (3, 2, 1) was a 3d_xz, which is unoccupied in every H–Ar
  // element and renders to a blank canvas for B–Ne under Slater
  // shielding (z_eff clamps to 0). 1s is occupied everywhere and never
  // empty. Element-change snap in `page.tsx` updates these to the
  // element's HOMO when the user picks a different atom.
  n: 1,
  l: 0,
  m: 0,
  colormap: 'INFERNO',
  autoRotate: false,
  hudVisible: true,
  useBareZ: false,
};

const ELEMENT_Z_MIN = 1;
const ELEMENT_Z_MAX = 18;

function clampElementZ(z: unknown): number {
  if (!Number.isFinite(z)) return DEFAULT_STORED_STATE.elementZ;
  const zi = Math.round(z as number);
  return Math.min(ELEMENT_Z_MAX, Math.max(ELEMENT_Z_MIN, zi));
}

const N_MIN = 1;
const N_MAX = 6;

function clampNlm(n: unknown, l: unknown, m: unknown): { n: number; l: number; m: number } {
  // Round + clamp `n` first; `l` depends on `n`; `m` depends on `l`.
  // Anything non-finite collapses to the default for that channel — we
  // don't want NaN propagating into the shader.
  const ni = Number.isFinite(n) ? Math.round(n as number) : DEFAULT_STORED_STATE.n;
  const nc = Math.min(N_MAX, Math.max(N_MIN, ni));

  const li = Number.isFinite(l) ? Math.round(l as number) : DEFAULT_STORED_STATE.l;
  const lc = Math.min(nc - 1, Math.max(0, li));

  const mi = Number.isFinite(m) ? Math.round(m as number) : 0;
  const mc = Math.min(lc, Math.max(-lc, mi));

  return { n: nc, l: lc, m: mc };
}

function coerceColormap(v: unknown): ColormapName {
  if (typeof v === 'string' && (COLORMAP_ORDER as readonly string[]).includes(v)) {
    return v as ColormapName;
  }
  return DEFAULT_STORED_STATE.colormap;
}

function coerceBool(v: unknown, fallback: boolean): boolean {
  return typeof v === 'boolean' ? v : fallback;
}

/**
 * Parse a raw JSON string (as written by `serializeStoredState`) into a
 * `StoredState`. Returns `null` if the input is missing or unparseable —
 * the caller should fall back to `DEFAULT_STORED_STATE`. Partial/garbled
 * objects are *not* rejected: each field is clamped or replaced
 * individually so a single bad key can't nuke the rest.
 */
export function parseStoredState(raw: string | null | undefined): StoredState | null {
  if (raw == null) return null;
  let obj: unknown;
  try {
    obj = JSON.parse(raw);
  } catch {
    return null;
  }
  if (typeof obj !== 'object' || obj === null) return null;
  const o = obj as Record<string, unknown>;

  const { n, l, m } = clampNlm(o.n, o.l, o.m);
  return {
    elementZ: clampElementZ(o.elementZ),
    n,
    l,
    m,
    colormap: coerceColormap(o.colormap),
    autoRotate: coerceBool(o.autoRotate, DEFAULT_STORED_STATE.autoRotate),
    hudVisible: coerceBool(o.hudVisible, DEFAULT_STORED_STATE.hudVisible),
    useBareZ: coerceBool(o.useBareZ, DEFAULT_STORED_STATE.useBareZ),
  };
}

export function serializeStoredState(s: StoredState): string {
  return JSON.stringify(s);
}

/**
 * Synchronously read + parse the stored state. Safe to call from
 * `useState` initializers because it guards `typeof window` — on the
 * server it just returns the defaults so SSR renders deterministically.
 */
export function loadStoredState(): StoredState {
  if (typeof window === 'undefined') return DEFAULT_STORED_STATE;
  try {
    const parsed = parseStoredState(window.localStorage.getItem(STORAGE_KEY));
    return parsed ?? DEFAULT_STORED_STATE;
  } catch {
    // localStorage can throw in private-browsing modes or when storage
    // is full; fall back to defaults rather than crash on mount.
    return DEFAULT_STORED_STATE;
  }
}

export function saveStoredState(s: StoredState): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(STORAGE_KEY, serializeStoredState(s));
  } catch {
    // Quota / private-browsing — swallow; persistence is best-effort.
  }
}
