// Debounce + cancel-in-flight state machine for the bake pipeline.
//
// Contract:
//   * Inputs change → wait 150ms of quiet, then fire one bake with the
//     latest params via the injected `client`.
//   * If a new change arrives while a bake is in flight, the in-flight
//     bake is cancelled (BakeClient.requestBake terminates the worker
//     internally) and a fresh one starts.
//   * `baking` is true from the moment we hand a request to the client
//     until the corresponding result lands (or is superseded).
//
// The client is injected so the hook can be unit-tested with a
// hand-rolled deferred mock; no real Worker required.

import { useEffect, useRef, useState } from 'react';

import { BakeCancelledError, type IBakeClient, type Volume } from '@/lib/bake/client';

const DEBOUNCE_MS = 150;

export type BakeInputs = {
  elementZ: number;
  n: number;
  l: number;
  m: number;
  useBareZ: boolean;
  res: number;
};

export type UseDebouncedBakeResult = {
  volume: Volume | null;
  baking: boolean;
};

export function useDebouncedBake(
  params: BakeInputs,
  client: IBakeClient,
): UseDebouncedBakeResult {
  const [volume, setVolume] = useState<Volume | null>(null);
  const [baking, setBaking] = useState(false);

  // Bump a generation token on every effect run so late-arriving
  // promise resolutions from superseded bakes are ignored.
  const genRef = useRef(0);

  useEffect(() => {
    const myGen = ++genRef.current;
    const timer = setTimeout(() => {
      if (genRef.current !== myGen) return;
      setBaking(true);
      client
        .requestBake(params)
        .then((v) => {
          if (genRef.current !== myGen) return;
          setVolume(v);
          setBaking(false);
        })
        .catch((err) => {
          if (genRef.current !== myGen) return;
          if (err instanceof BakeCancelledError) return;
          // Unknown failure — surface via `baking=false` so the UI
          // doesn't get stuck. Throwing here would crash the React tree.
          setBaking(false);
        });
    }, DEBOUNCE_MS);

    return () => {
      clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [params.elementZ, params.n, params.l, params.m, params.useBareZ, params.res, client]);

  return { volume, baking };
}
