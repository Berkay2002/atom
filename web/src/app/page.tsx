'use client';

import { useState } from 'react';

import AtomCanvas from '@/components/AtomCanvas';
import Controls, { type OrbitalParams } from '@/components/Controls';
import type { ColormapName } from '@/lib/colormaps';

// Default matches `atom-desktop` UiState::default(): 3d_(xy)-ish lobe so
// first-load doesn't look like a boring 1s sphere.
const DEFAULT_PARAMS: OrbitalParams = { n: 3, l: 2, m: 1 };

// Inferno is the desktop default too — bright fiery palette reads well on
// the dark backdrop.
const DEFAULT_COLORMAP: ColormapName = 'INFERNO';

export default function Home() {
  const [params, setParams] = useState<OrbitalParams>(DEFAULT_PARAMS);
  const [colormap, setColormap] = useState<ColormapName>(DEFAULT_COLORMAP);
  return (
    <>
      <AtomCanvas params={params} colormap={colormap} />
      <Controls
        value={params}
        onChange={setParams}
        colormap={colormap}
        onColormapChange={setColormap}
      />
    </>
  );
}
