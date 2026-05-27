import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import init, {
  scene_decode_projection,
  scene_encode_projection,
} from '../../wasm/atom_core.js';

import { decodeScene, encodeScene, SceneDecodeError } from './scene-url';

describe('scene URL projection codec', () => {
  beforeAll(async () => {
    await init(await readFile(new URL('../../wasm/atom_core_bg.wasm', import.meta.url)));
  });

  it('encodes the fixed fidelity case through the browser projection', () => {
    expect(
      scene_encode_projection({
        elementZ: 1,
        n: 1,
        l: 0,
        m: 0,
        useBareZ: false,
        colormapId: 0,
        exposure: 1,
      }),
    ).toBe('v1:1/1/0/0/eff/0/1.00');
  });

  it('decodes the fixed fidelity case to the browser projection shape', () => {
    expect(scene_decode_projection('v1:1/1/0/0/eff/0/1.00')).toEqual({
      elementZ: 1,
      n: 1,
      l: 0,
      m: 0,
      useBareZ: false,
      colormapId: 0,
      exposure: 1,
    });
  });

  it('round-trips representative web state through the TypeScript adapter', () => {
    const state = {
      elementZ: 8,
      n: 2,
      l: 1,
      m: -1,
      useBareZ: true,
      colormap: 'PLASMA' as const,
      exposure: 1.25,
    };

    expect(decodeScene(encodeScene(state))).toEqual(state);
  });

  it('keeps decode errors readable for the TypeScript adapter', () => {
    expect(() => decodeScene('v2:1/1/0/0/eff/0/1.00')).toThrow(SceneDecodeError);
    expect(() => decodeScene('v2:1/1/0/0/eff/0/1.00')).toThrow("unsupported version 'v2'");
  });
});
