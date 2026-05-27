'use client';

import { useState } from 'react';

import AtomCanvas from '@/components/AtomCanvas';
import Controls, { type OrbitalParams } from '@/components/Controls';

// Default matches `atom-desktop` UiState::default(): 3d_(xy)-ish lobe so
// first-load doesn't look like a boring 1s sphere.
const DEFAULT_PARAMS: OrbitalParams = { n: 3, l: 2, m: 1 };

export default function Home() {
  const [params, setParams] = useState<OrbitalParams>(DEFAULT_PARAMS);
  return (
    <>
      <AtomCanvas params={params} />
      <Controls value={params} onChange={setParams} />
    </>
  );
}
