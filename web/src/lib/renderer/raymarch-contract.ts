import type { ColormapStops } from '../colormaps';

const LUT_ENTRIES = 256;
const LUT_CHANNELS = 4;
const LUT_ALPHA = 255;

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}

export function raymarchIntensity(sum: number, k: number, exposure: number): number {
  return clamp01(exposure * (1 - Math.exp(-k * sum)));
}

export function buildColormapLutBytes(stops: ColormapStops): Uint8Array {
  const n = stops.length;
  if (n < 2) throw new Error(`colormap needs at least 2 stops, got ${n}`);

  const data = new Uint8Array(LUT_ENTRIES * LUT_CHANNELS);
  for (let i = 0; i < LUT_ENTRIES; i += 1) {
    const t = i / (LUT_ENTRIES - 1);
    const f = t * (n - 1);
    const lo = Math.floor(f);
    const hi = Math.min(lo + 1, n - 1);
    const a = f - lo;
    const inv = 1 - a;
    const c0 = stops[lo];
    const c1 = stops[hi];
    const off = i * LUT_CHANNELS;
    data[off] = Math.round(c0[0] * inv + c1[0] * a);
    data[off + 1] = Math.round(c0[1] * inv + c1[1] * a);
    data[off + 2] = Math.round(c0[2] * inv + c1[2] * a);
    data[off + 3] = LUT_ALPHA;
  }

  return data;
}
