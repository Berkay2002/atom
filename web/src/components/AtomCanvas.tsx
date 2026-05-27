'use client';

// Glue between the bake worker, the WebGL2 renderer, the orbit camera,
// and the DOM canvas.
//
//   * spawn the bake worker once on mount
//   * request a single bake(1, 0, 0, 96)
//   * once the volume arrives, call camera.fit(halfExtent)
//   * each rAF tick: rebuild the view-projection and feed the renderer
//   * pointer / wheel / touch are routed to the camera

import { useEffect, useRef } from 'react';
import { mat4 } from 'gl-matrix';

import { OrbitCamera } from '@/lib/camera/orbit-camera';
import { attachPointerInput } from '@/lib/camera/pointer-input';
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

    // Aspect is updated on the first sizeToWindow call below, so the
    // initial 1.0 is just a placeholder. Radius is meaningless until the
    // first bake fires camera.fit(halfExtent).
    const camera = new OrbitCamera(2.0, 1.0);

    let rafId = 0;
    let disposed = false;

    const sizeToWindow = () => {
      const dpr = window.devicePixelRatio || 1;
      const w = Math.floor(canvas.clientWidth * dpr);
      const h = Math.floor(canvas.clientHeight * dpr);
      renderer.resize(w, h);
      camera.aspect = Math.max(1, w) / Math.max(1, h);
    };
    sizeToWindow();
    window.addEventListener('resize', sizeToWindow);

    const detachInput = attachPointerInput(canvas, camera);

    // The worker URL pattern is how Next.js (webpack + turbopack) picks
    // up a TS file as a Web Worker entry and bundles it as its own chunk.
    const worker = new Worker(new URL('../lib/bake-worker.ts', import.meta.url), {
      type: 'module',
    });

    // Reused per-frame to avoid allocating a fresh Float32Array every tick.
    const invVP = mat4.create();

    worker.addEventListener('message', (ev: MessageEvent<WorkerMsg>) => {
      const msg = ev.data;
      if (msg.type === 'ready') {
        worker.postMessage({ type: 'bake', n: N, l: L, m: M, res: RES });
      } else if (msg.type === 'bake-result') {
        renderer.setVolume({ data: msg.data, res: RES, halfExtent: msg.halfExtent });
        camera.fit(msg.halfExtent);
      }
    });

    const tick = () => {
      if (disposed) return;
      const vp = camera.viewProj();
      mat4.invert(invVP, vp);
      renderer.setCamera({
        position: camera.position(),
        invViewProj: invVP as Float32Array,
      });
      renderer.draw();
      rafId = requestAnimationFrame(tick);
    };
    rafId = requestAnimationFrame(tick);

    return () => {
      disposed = true;
      cancelAnimationFrame(rafId);
      window.removeEventListener('resize', sizeToWindow);
      detachInput();
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
