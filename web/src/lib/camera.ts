// Fixed-orbit camera math for the tracer-bullet slice.
//
// Mirrors `crates/atom-desktop/src/camera.rs`:
//   pos = (r cos(el) sin(az), r sin(el), r cos(el) cos(az))
//   proj = perspective_rh(fovY, aspect, 0.1, 4r)
//   view = lookAt_rh(pos, origin, +Y)
//
// For slice 03 there is no interaction — pos/azimuth/elevation never
// change. Callers build a fresh CameraState from `view(half, aspect)`
// each time the canvas resizes or the volume re-bakes.

import { mat4, vec3 } from 'gl-matrix';

const AZIMUTH = (45 * Math.PI) / 180;
const ELEVATION = (30 * Math.PI) / 180;
const FOV_Y = (60 * Math.PI) / 180;

export type CameraView = {
  position: [number, number, number];
  invViewProj: Float32Array;
};

export function fixedView(halfExtent: number, aspect: number): CameraView {
  const radius = 2.0 * halfExtent;
  const px = radius * Math.cos(ELEVATION) * Math.sin(AZIMUTH);
  const py = radius * Math.sin(ELEVATION);
  const pz = radius * Math.cos(ELEVATION) * Math.cos(AZIMUTH);

  const proj = mat4.create();
  mat4.perspectiveNO(proj, FOV_Y, aspect, 0.1, radius * 4.0);

  const view = mat4.create();
  mat4.lookAt(view, vec3.fromValues(px, py, pz), vec3.fromValues(0, 0, 0), vec3.fromValues(0, 1, 0));

  const vp = mat4.create();
  mat4.multiply(vp, proj, view);

  const inv = mat4.create();
  mat4.invert(inv, vp);

  // gl-matrix mat4 is a Float32Array under the hood (length 16, column-major)
  // — exactly what WebGL's uniformMatrix4fv expects.
  return {
    position: [px, py, pz],
    invViewProj: new Float32Array(inv),
  };
}
