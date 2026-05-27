// Orbit camera around the origin. World units = a₀.
//
// Pure math, no DOM dependencies. Ports
// `crates/atom-desktop/src/camera.rs` line-for-line:
//   pos = (r cos(el) sin(az), r sin(el), r cos(el) cos(az))
//   proj = perspective_rh(fovY, aspect, 0.1, 4r)
//   view = lookAt_rh(pos, origin, +Y)
//   viewProj = proj * view
//
// Interaction:
//   * orbit(dxPx, dyPx) — drag in screen pixels, SENS = 0.005 rad/px;
//     elevation clamps to ±(π/2 − 0.01) to dodge the gimbal flip.
//   * zoom(factor) — multiplies radius; floor of 0.1 keeps the eye
//     from punching through the origin.
//   * fit(halfExtent) — snaps radius to 2 × halfExtent and resets
//     azimuth/elevation to the canonical preview pose. fov_y and
//     aspect are left untouched (matches the desktop).

import { mat4, vec3 } from 'gl-matrix';

const SENS = 0.005;
const ELEVATION_LIMIT = Math.PI / 2 - 0.01;
const RADIUS_FLOOR = 0.1;

const DEFAULT_AZIMUTH = (45 * Math.PI) / 180;
const DEFAULT_ELEVATION = (30 * Math.PI) / 180;
const DEFAULT_FOV_Y = (60 * Math.PI) / 180;

export class OrbitCamera {
  radius: number;
  azimuth: number;
  elevation: number;
  fovY: number;
  aspect: number;

  constructor(radius: number, aspect: number) {
    this.radius = radius;
    this.azimuth = DEFAULT_AZIMUTH;
    this.elevation = DEFAULT_ELEVATION;
    this.fovY = DEFAULT_FOV_Y;
    this.aspect = aspect;
  }

  position(): [number, number, number] {
    const r = this.radius;
    const ce = Math.cos(this.elevation);
    const se = Math.sin(this.elevation);
    const ca = Math.cos(this.azimuth);
    const sa = Math.sin(this.azimuth);
    return [r * ce * sa, r * se, r * ce * ca];
  }

  viewProj(): Float32Array {
    const proj = mat4.create();
    // Use perspectiveNO so the depth range matches OpenGL/wgpu RH (-1..1)
    // — same convention as glam's perspective_rh.
    mat4.perspectiveNO(proj, this.fovY, this.aspect, 0.1, this.radius * 4.0);

    const view = mat4.create();
    const [px, py, pz] = this.position();
    mat4.lookAt(
      view,
      vec3.fromValues(px, py, pz),
      vec3.fromValues(0, 0, 0),
      vec3.fromValues(0, 1, 0)
    );

    const vp = mat4.create();
    mat4.multiply(vp, proj, view);
    return vp as Float32Array;
  }

  /** Snap radius/angles to frame an orbital with box half-extent `half`. */
  fit(half: number): void {
    this.radius = 2.0 * half;
    this.elevation = DEFAULT_ELEVATION;
    this.azimuth = DEFAULT_AZIMUTH;
  }

  /** Apply a mouse/touch drag in pixels. */
  orbit(dxPx: number, dyPx: number): void {
    this.azimuth += dxPx * SENS;
    this.elevation = clamp(
      this.elevation - dyPx * SENS,
      -ELEVATION_LIMIT,
      ELEVATION_LIMIT
    );
  }

  /** Apply a wheel/pinch delta; `factor > 1` zooms out, `< 1` zooms in. */
  zoom(factor: number): void {
    this.radius = Math.max(this.radius * factor, RADIUS_FLOOR);
  }
}

function clamp(x: number, lo: number, hi: number): number {
  return Math.min(Math.max(x, lo), hi);
}
