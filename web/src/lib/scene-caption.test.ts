import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import init, {
  scene_caption_projection,
} from '../../wasm/atom_core.js';
import { sceneCaption } from './scene-caption';

describe('scene caption projection', () => {
  beforeAll(async () => {
    await init(await readFile(new URL('../../wasm/atom_core_bg.wasm', import.meta.url)));
  });

  it('matches the shared Rust caption through the browser projection shape', () => {
    const projection = {
      elementZ: 6,
      n: 2,
      l: 1,
      m: -1,
      useBareZ: false,
      colormapId: 0,
      exposure: 1,
    };

    expect(scene_caption_projection(projection)).toBe(
      'Carbon 2p_y — a dumbbell-shaped orbital with two lobes along one axis.',
    );
    expect(sceneCaption(projection)).toBe(scene_caption_projection(projection));
  });
});
