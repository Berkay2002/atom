'use client';

// Glue between the bake pipeline (BakeClient + useDebouncedBake), the
// WebGL2 renderer, the orbit camera, and the DOM canvas.
//
//   * one BakeClient per mount; it lazily spawns + tears down workers
//   * useDebouncedBake turns rapid (n, l, m) changes into a single bake
//   * each new volume → renderer.setVolume + camera.fit(halfExtent)
//   * pointer / wheel / touch are routed to the camera

import { useEffect, useMemo, useRef } from 'react';
import { mat4 } from 'gl-matrix';

import { OrbitCamera } from '@/lib/camera/orbit-camera';
import { attachPointerInput } from '@/lib/camera/pointer-input';
import { Raymarcher } from '@/lib/renderer/raymarch';
import { BakeClient } from '@/lib/bake/client';
import { COLORMAPS, type ColormapName } from '@/lib/colormaps';
import { useDebouncedBake } from '@/hooks/useDebouncedBake';
import type { OrbitalParams } from './Controls';

const RES = 96;

// Tracer-bullet defaults that match `atom-desktop` initial UI values.
const RAYMARCH_PARAMS = { k: 5, exposure: 1, steps: 256 };

export type AtomCanvasProps = {
  params: OrbitalParams;
  colormap: ColormapName;
};

export default function AtomCanvas({ params, colormap }: AtomCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const rendererRef = useRef<Raymarcher | null>(null);
  const cameraRef = useRef<OrbitCamera | null>(null);

  // One BakeClient for the lifetime of this component instance. The
  // factory creates a fresh module Worker per requestBake — cancellation
  // is terminate-and-respawn, owned inside the client.
  const client = useMemo(
    () =>
      new BakeClient(
        () =>
          new Worker(new URL('../lib/bake-worker.ts', import.meta.url), {
            type: 'module',
          }),
      ),
    [],
  );

  const bakeParams = useMemo(
    () => ({ n: params.n, l: params.l, m: params.m, res: RES }),
    [params.n, params.l, params.m],
  );
  const { volume } = useDebouncedBake(bakeParams, client);

  useEffect(() => {
    return () => {
      client.dispose();
    };
  }, [client]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const renderer = new Raymarcher(canvas);
    renderer.setParams(RAYMARCH_PARAMS);
    // Upload the initial LUT before the first frame so we never flash
    // grayscale / uninitialized colour. The colormap effect below also
    // syncs on prop change, but seeding here avoids a 1-frame gap when
    // the initial value happens to be the React default.
    renderer.setColormap(COLORMAPS[colormap]);
    rendererRef.current = renderer;

    // Aspect is updated on the first sizeToWindow call below, so the
    // initial 1.0 is just a placeholder. Radius is reset by camera.fit
    // when the first volume arrives.
    const camera = new OrbitCamera(2.0, 1.0);
    cameraRef.current = camera;

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

    // Reused per-frame to avoid allocating a fresh Float32Array every tick.
    const invVP = mat4.create();

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
      renderer.dispose();
      rendererRef.current = null;
      cameraRef.current = null;
    };
  }, []);

  // Each new volume → push to GPU and reframe. Re-fitting on every bake
  // (not just the first) keeps wildly different orbital sizes — say a 1s
  // vs a 6f — both nicely framed when the user scrubs.
  useEffect(() => {
    if (!volume) return;
    const renderer = rendererRef.current;
    const camera = cameraRef.current;
    if (!renderer || !camera) return;
    renderer.setVolume({
      data: volume.data,
      res: volume.res,
      halfExtent: volume.halfExtent,
    });
    camera.fit(volume.halfExtent);
  }, [volume]);

  // Swap the LUT in place when the picker selection changes. No bake
  // request — the volume hasn't moved; the rAF loop just picks up the
  // new palette on the next draw. Skipped on first mount when the
  // renderer ref isn't populated yet (the initial seed happens inside
  // the mount effect above).
  useEffect(() => {
    const renderer = rendererRef.current;
    if (!renderer) return;
    renderer.setColormap(COLORMAPS[colormap]);
  }, [colormap]);

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
