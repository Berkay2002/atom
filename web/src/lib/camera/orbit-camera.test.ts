// Ports the three #[test] blocks from
// `crates/atom-desktop/src/camera.rs`. These are the only tests required
// for the web camera slice — the rest of the behavior is verified
// visually in the browser, same as the desktop app.

import { describe, expect, it } from 'vitest';

import { OrbitCamera } from './orbit-camera';

const HALF_PI = Math.PI / 2;
const EPS = 1e-5;

function close(a: number, b: number): void {
  expect(Math.abs(a - b)).toBeLessThan(EPS);
}

describe('OrbitCamera', () => {
  it('fit() sets radius to twice the half-extent and resets angles', () => {
    const c = new OrbitCamera(1.0, 1.0);
    c.fit(12.0);
    close(c.radius, 24.0);
    close(c.elevation, (30 * Math.PI) / 180);
    close(c.azimuth, (45 * Math.PI) / 180);
  });

  it('elevation clamps just shy of ±π/2 under extreme drag', () => {
    const c = new OrbitCamera(10.0, 1.0);
    c.elevation = 0;
    c.orbit(0, -10_000);
    expect(c.elevation).toBeLessThan(HALF_PI);
    c.orbit(0, 10_000);
    expect(c.elevation).toBeGreaterThan(-HALF_PI);
  });

  it('zoom() multiplies radius', () => {
    const c = new OrbitCamera(10.0, 1.0);
    c.zoom(2.0);
    close(c.radius, 20.0);
    c.zoom(0.25);
    close(c.radius, 5.0);
  });
});
