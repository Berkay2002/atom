// Guided-tour types + loaders + URL helpers (issue 07).
//
// A tour is a static JSON file under `web/public/tours/<slug>.json` and an
// `index.json` that lists which slugs are available. Both are fetched on
// demand: there is no build-time bundling, so an author can drop a new tour
// in and update the index without rebuilding the app.
//
// The tour *engine* lives at the page level — this module is just the data
// layer (types + fetch + parse + URL-param round-trip).

const TOURS_DIR = '/tours';

/** Search-param keys for tour mode. URL shape: `?tour=<slug>&step=<n>`. */
export const TOUR_SLUG_PARAM = 'tour';
export const TOUR_STEP_PARAM = 'step';

/** A single step in a tour: a Scene URL string + the caption to display. */
export type TourStep = {
  /** Scene URL token (the `v1:...` payload) — same shape `decodeScene` takes. */
  scene: string;
  /** Sentence(s) shown in the caption bar while this step is active. */
  caption: string;
};

/** The full deserialized tour, as authored in `<slug>.json`. */
export type Tour = {
  /** Filename slug (e.g. `shielding-period-2`). Set by the loader, not by the file. */
  slug: string;
  /** Human-facing name shown in the tour picker and as the tour title in-mode. */
  name: string;
  /** One-paragraph description, shown in the picker as a hover preview. */
  description: string;
  steps: TourStep[];
};

/** A single entry in `index.json`. The full tour body still has to be fetched. */
export type TourIndexEntry = {
  slug: string;
  name: string;
  description: string;
};

/** Throw when a tour file or index file is missing / malformed. */
export class TourLoadError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'TourLoadError';
  }
}

/**
 * Fetch the tour index. Cached for the lifetime of the page — the index is
 * effectively a static manifest so refetching on every picker open would be
 * wasteful.
 */
let indexCache: Promise<TourIndexEntry[]> | null = null;
export function loadTourIndex(): Promise<TourIndexEntry[]> {
  if (!indexCache) {
    indexCache = fetchAndParseIndex();
  }
  return indexCache;
}

async function fetchAndParseIndex(): Promise<TourIndexEntry[]> {
  const url = `${TOURS_DIR}/index.json`;
  let res: Response;
  try {
    res = await fetch(url);
  } catch (err) {
    throw new TourLoadError(`Couldn't reach ${url}: ${err instanceof Error ? err.message : String(err)}`);
  }
  if (!res.ok) {
    throw new TourLoadError(`Tour index missing (HTTP ${res.status})`);
  }
  let body: unknown;
  try {
    body = await res.json();
  } catch {
    throw new TourLoadError('Tour index is not valid JSON.');
  }
  return parseIndex(body);
}

/** Pure parser. Exposed so unit tests can hit it without a fetch. */
export function parseIndex(body: unknown): TourIndexEntry[] {
  if (!isObject(body) || !Array.isArray((body as Record<string, unknown>).tours)) {
    throw new TourLoadError('Tour index must have a `tours` array.');
  }
  const raw = (body as { tours: unknown[] }).tours;
  const entries: TourIndexEntry[] = [];
  raw.forEach((entry, i) => {
    if (!isObject(entry)) {
      throw new TourLoadError(`Tour index entry ${i} is not an object.`);
    }
    const o = entry as Record<string, unknown>;
    if (typeof o.slug !== 'string' || o.slug.length === 0) {
      throw new TourLoadError(`Tour index entry ${i} is missing a string \`slug\`.`);
    }
    if (typeof o.name !== 'string' || o.name.length === 0) {
      throw new TourLoadError(`Tour ${o.slug} is missing a string \`name\`.`);
    }
    if (typeof o.description !== 'string') {
      throw new TourLoadError(`Tour ${o.slug} is missing a string \`description\`.`);
    }
    entries.push({ slug: o.slug, name: o.name, description: o.description });
  });
  return entries;
}

/** Fetch a single tour by slug. No caching — tours are small and the user
 *  navigates between them rarely, so a stale cache isn't worth the bytes. */
export async function loadTour(slug: string): Promise<Tour> {
  if (!/^[a-z0-9-]+$/i.test(slug)) {
    // Reject slugs that could escape the tours directory or contain
    // surprising characters. The picker only emits sanitized slugs, but
    // the URL is user-controllable.
    throw new TourLoadError(`Tour slug "${slug}" contains invalid characters.`);
  }
  const url = `${TOURS_DIR}/${slug}.json`;
  let res: Response;
  try {
    res = await fetch(url);
  } catch (err) {
    throw new TourLoadError(`Couldn't reach ${url}: ${err instanceof Error ? err.message : String(err)}`);
  }
  if (!res.ok) {
    throw new TourLoadError(`Tour "${slug}" not found (HTTP ${res.status}).`);
  }
  let body: unknown;
  try {
    body = await res.json();
  } catch {
    throw new TourLoadError(`Tour "${slug}" is not valid JSON.`);
  }
  return parseTour(slug, body);
}

/** Pure parser. Exposed so unit tests can hit it without a fetch. */
export function parseTour(slug: string, body: unknown): Tour {
  if (!isObject(body)) {
    throw new TourLoadError(`Tour "${slug}" body must be an object.`);
  }
  const o = body as Record<string, unknown>;
  if (typeof o.name !== 'string' || o.name.length === 0) {
    throw new TourLoadError(`Tour "${slug}" is missing a string \`name\`.`);
  }
  if (typeof o.description !== 'string') {
    throw new TourLoadError(`Tour "${slug}" is missing a string \`description\`.`);
  }
  if (!Array.isArray(o.steps) || o.steps.length === 0) {
    throw new TourLoadError(`Tour "${slug}" must have a non-empty \`steps\` array.`);
  }
  const steps: TourStep[] = o.steps.map((step, i) => {
    if (!isObject(step)) {
      throw new TourLoadError(`Tour "${slug}" step ${i} is not an object.`);
    }
    const s = step as Record<string, unknown>;
    if (typeof s.scene !== 'string' || s.scene.length === 0) {
      throw new TourLoadError(`Tour "${slug}" step ${i} is missing a string \`scene\`.`);
    }
    if (typeof s.caption !== 'string' || s.caption.length === 0) {
      throw new TourLoadError(`Tour "${slug}" step ${i} is missing a string \`caption\`.`);
    }
    return { scene: s.scene, caption: s.caption };
  });
  return { slug, name: o.name, description: o.description, steps };
}

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null;
}

/**
 * Read tour params from a URL search string. Returns `null` when either
 * param is missing, malformed, or zero/negative. The step is 0-indexed
 * internally but 1-indexed in the URL (more natural to a human typing one).
 */
export function readTourParams(
  searchString: string | null | undefined,
): { slug: string; step: number } | null {
  if (!searchString) return null;
  const stripped = searchString.startsWith('?') ? searchString.slice(1) : searchString;
  const params = new URLSearchParams(stripped);
  const slug = params.get(TOUR_SLUG_PARAM);
  const stepRaw = params.get(TOUR_STEP_PARAM);
  if (!slug || slug.length === 0) return null;
  const stepNum = stepRaw ? Number.parseInt(stepRaw, 10) : 1;
  if (!Number.isFinite(stepNum) || stepNum < 1) return null;
  return { slug, step: stepNum - 1 };
}

/**
 * Build a new search params object with `tour=...&step=...` set (step is
 * stored 1-indexed). Any pre-existing `?s=` is dropped — when a tour is
 * active the URL is authoritative through the tour params instead.
 */
export function withTourParams(
  currentSearch: string | null | undefined,
  slug: string,
  step: number,
): URLSearchParams {
  const stripped = currentSearch?.startsWith('?')
    ? currentSearch.slice(1)
    : currentSearch ?? '';
  const params = new URLSearchParams(stripped);
  params.set(TOUR_SLUG_PARAM, slug);
  params.set(TOUR_STEP_PARAM, String(step + 1));
  // Drop `?s=` while tour is active so the URL only encodes one "source of
  // truth" for the scene. On exit, the page re-adds `?s=` for the final view.
  params.delete('s');
  return params;
}

/** Remove tour params from the URL — used when the user clicks Exit. */
export function clearTourParams(
  currentSearch: string | null | undefined,
): URLSearchParams {
  const stripped = currentSearch?.startsWith('?')
    ? currentSearch.slice(1)
    : currentSearch ?? '';
  const params = new URLSearchParams(stripped);
  params.delete(TOUR_SLUG_PARAM);
  params.delete(TOUR_STEP_PARAM);
  return params;
}

/** Test-only hook to drop the index cache (vitest spies on `fetch`). */
export function __resetTourIndexCache(): void {
  indexCache = null;
}
