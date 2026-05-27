// Pointer Events + wheel adapter for OrbitCamera.
//
// One unified handler for mouse, pen, and touch via the Pointer Events
// API. Single-pointer drag drives `orbit`; two-pointer pinch drives
// `zoom` using the ratio of inter-pointer distances. The wheel handler
// maps wheel deltas through an exponential to match the desktop's
// "feels right" zoom rate.
//
// No React, no state outside the closure — call `attachPointerInput`
// once from a useEffect and call the returned `detach` on cleanup.

import type { OrbitCamera } from './orbit-camera';

const WHEEL_SENSITIVITY = 0.001;

type ActivePointer = { x: number; y: number };

export function attachPointerInput(
  canvas: HTMLCanvasElement,
  camera: OrbitCamera
): () => void {
  const active = new Map<number, ActivePointer>();
  // Inter-pointer distance from the previous two-pointer frame. Reset
  // whenever the pointer count changes so the next pinch sample is
  // baseline rather than a phantom jump.
  let lastPinchDistance: number | null = null;

  const onPointerDown = (e: PointerEvent) => {
    canvas.setPointerCapture(e.pointerId);
    active.set(e.pointerId, { x: e.clientX, y: e.clientY });
    lastPinchDistance = null;
    e.preventDefault();
  };

  const onPointerMove = (e: PointerEvent) => {
    const prev = active.get(e.pointerId);
    if (!prev) return;

    const next = { x: e.clientX, y: e.clientY };
    active.set(e.pointerId, next);

    if (active.size === 1) {
      const dx = next.x - prev.x;
      const dy = next.y - prev.y;
      camera.orbit(dx, dy);
    } else if (active.size === 2) {
      const [a, b] = [...active.values()];
      const dist = Math.hypot(a.x - b.x, a.y - b.y);
      if (lastPinchDistance !== null && lastPinchDistance > 0 && dist > 0) {
        // Spreading fingers (dist grows) zooms in → factor < 1.
        const factor = lastPinchDistance / dist;
        camera.zoom(factor);
      }
      lastPinchDistance = dist;
    }
    e.preventDefault();
  };

  const releasePointer = (e: PointerEvent) => {
    if (!active.has(e.pointerId)) return;
    active.delete(e.pointerId);
    lastPinchDistance = null;
    if (canvas.hasPointerCapture(e.pointerId)) {
      canvas.releasePointerCapture(e.pointerId);
    }
  };

  const onWheel = (e: WheelEvent) => {
    // Up-scroll (deltaY < 0) zooms in; down-scroll zooms out. Clamp the
    // exponent so a single trackpad fling can't blow the radius up by
    // orders of magnitude in one frame.
    const clamped = Math.max(-200, Math.min(200, e.deltaY));
    const factor = Math.exp(clamped * WHEEL_SENSITIVITY);
    camera.zoom(factor);
    e.preventDefault();
  };

  // `{ passive: false }` is required so preventDefault on wheel and
  // touch-initiated pointer events actually stops the page from
  // scrolling underneath the canvas on mobile.
  canvas.addEventListener('pointerdown', onPointerDown);
  canvas.addEventListener('pointermove', onPointerMove);
  canvas.addEventListener('pointerup', releasePointer);
  canvas.addEventListener('pointercancel', releasePointer);
  canvas.addEventListener('wheel', onWheel, { passive: false });
  // Disable the right-click menu so a future right-drag binding has
  // somewhere to live, and the page never pops a menu mid-orbit.
  const onContextMenu = (e: Event) => e.preventDefault();
  canvas.addEventListener('contextmenu', onContextMenu);
  // Touch-action: none on the element-level CSS would also work, but
  // setting it via style here keeps the camera module self-contained.
  const prevTouchAction = canvas.style.touchAction;
  canvas.style.touchAction = 'none';

  return () => {
    canvas.removeEventListener('pointerdown', onPointerDown);
    canvas.removeEventListener('pointermove', onPointerMove);
    canvas.removeEventListener('pointerup', releasePointer);
    canvas.removeEventListener('pointercancel', releasePointer);
    canvas.removeEventListener('wheel', onWheel);
    canvas.removeEventListener('contextmenu', onContextMenu);
    canvas.style.touchAction = prevTouchAction;
    active.clear();
  };
}
