'use client';

// Glue between the bake worker, the WebGL2 renderer, and the DOM canvas.
//
// Slice 03 is intentionally minimal:
//   * spawn the bake worker once on mount
//   * request a single bake(1, 0, 0, 96)
//   * upload the volume, build one fixed camera view, and run a clean
//     requestAnimationFrame loop
//
// No state, no controls. Resizing the window is the only dynamic input.

import { useEffect, useRef } from 'react';

import { fixedView } from '@/lib/camera';
import { Raymarcher } from '@/lib/renderer/raymarch';

const N = 1;
const L = 0;
const M = 0;
const RES = 96;

// Tracer-bullet defaults that match `atom-desktop` initial UI values.
const RAYMARCH_PARAMS = { k: 5, exposure: 1, steps: 256 };

type BakeResultMsg = {
  type: 'bake-result';
  data: Float32Array;
  halfExtent: number;
  peak: number;
};

type WorkerMsg = { type: 'ready' } | BakeResultMsg;

export default function AtomCanvas() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const renderer = new Raymarcher(canvas);
    renderer.setParams(RAYMARCH_PARAMS);

    let halfExtent = 1;
    let needsCameraUpdate = true;
    let rafId = 0;
    let disposed = false;

    const sizeToWindow = () => {
      const dpr = window.devicePixelRatio || 1;
      const w = Math.floor(canvas.clientWidth * dpr);
      const h = Math.floor(canvas.clientHeight * dpr);
      renderer.resize(w, h);
      needsCameraUpdate = true;
    };
    sizeToWindow();
    window.addEventListener('resize', sizeToWindow);

    // The worker URL pattern is how Next.js (webpack + turbopack) picks
    // up a TS file as a Web Worker entry and bundles it as its own chunk.
    const worker = new Worker(new URL('../lib/bake-worker.ts', import.meta.url), {
      type: 'module',
    });

    worker.addEventListener('message', (ev: MessageEvent<WorkerMsg>) => {
      const msg = ev.data;
      if (msg.type === 'ready') {
        worker.postMessage({ type: 'bake', n: N, l: L, m: M, res: RES });
      } else if (msg.type === 'bake-result') {
        renderer.setVolume({ data: msg.data, res: RES, halfExtent: msg.halfExtent });
        halfExtent = msg.halfExtent;
        needsCameraUpdate = true;
      }
    });

    const tick = () => {
      if (disposed) return;
      if (needsCameraUpdate) {
        const aspect = Math.max(1, canvas.width) / Math.max(1, canvas.height);
        const cam = fixedView(halfExtent, aspect);
        renderer.setCamera(cam);
        needsCameraUpdate = false;
      }
      renderer.draw();
      rafId = requestAnimationFrame(tick);
    };
    rafId = requestAnimationFrame(tick);

    return () => {
      disposed = true;
      cancelAnimationFrame(rafId);
      window.removeEventListener('resize', sizeToWindow);
      worker.terminate();
      renderer.dispose();
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      style={{
        position: 'fixed',
        inset: 0,
        width: '100vw',
        height: '100vh',
        display: 'block',
        background: '#000',
      }}
    />
  );
}
