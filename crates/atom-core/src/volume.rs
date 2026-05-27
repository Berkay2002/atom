//! Adaptive cartesian box around the scene, sized to fit every atom's
//! orbital. Cubic box of edge length 6·n²·a₀ per atom; `box_extent` returns
//! the half-edge (radius) for a single orbital. `bake_scene` evaluates the
//! summed density `Σ |ψ_atom|²` on a regular grid and peak-normalizes
//! across the whole volume. On native targets the evaluation is
//! rayon-parallelized; on `wasm32` it falls back to a plain serial iterator
//! (rayon needs atomics that aren't in our wasm32-unknown-unknown target).

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

use crate::scene::{Atom, Orbital, Scene};

/// Half-edge of the cubic bounding box for an orbital with principal quantum
/// number `n`. Returned in atomic units (a₀).
pub fn box_extent(n: u32) -> f64 {
    3.0 * (n as f64).powi(2)
}

/// Per-atom orbital "radius" used by the adaptive box sizing. Today this is
/// just `box_extent(orbital.n)`; once `z_eff` lands (issue 02) it will
/// shrink with increasing nuclear charge.
fn atom_radius(orbital: Orbital) -> f64 {
    box_extent(orbital.n)
}

/// Baked volume: peak-normalized density on a cubic grid centered at the
/// scene origin.
pub struct Volume {
    pub data: Vec<f32>,   // length = res³, row-major, x fastest then y then z
    pub res: usize,
    pub half_extent: f64, // a₀
    pub peak: f64,        // absolute peak Σ|ψ_atom|² before normalization (for HUD)
}

/// Bake the volume for `scene` at the given grid resolution.
///
/// The bounding box is the smallest cube centered at the scene origin that
/// contains every atom's `|position| + atom_radius(orbital)`. For a single
/// atom at the origin this collapses to today's `3·n²·a₀` half-edge.
///
/// Per voxel, densities from every atom are summed (`Σ |ψ_atom|²`), then
/// the whole grid is peak-normalized to `[0, 1]`. The absolute peak is
/// preserved in `Volume::peak`.
///
/// Hydrogen-only for slice 1: `z_eff` is hard-coded to `1.0` (bare hydrogen).
pub fn bake_scene(scene: &Scene, res: usize) -> Volume {
    let half_extent = scene
        .atoms
        .iter()
        .map(|a| {
            let r = (a.position[0] * a.position[0]
                + a.position[1] * a.position[1]
                + a.position[2] * a.position[2])
                .sqrt();
            r + atom_radius(a.orbital)
        })
        .fold(0.0_f64, f64::max);
    let step = 2.0 * half_extent / res as f64;
    let total = res * res * res;

    let atoms: &[Atom] = &scene.atoms;
    let sample = |idx: usize| -> f64 {
        let i = idx % res;
        let j = (idx / res) % res;
        let k = idx / (res * res);
        let x = -half_extent + (i as f64 + 0.5) * step;
        let y = -half_extent + (j as f64 + 0.5) * step;
        let z = -half_extent + (k as f64 + 0.5) * step;
        let mut sum = 0.0_f64;
        for a in atoms {
            let dx = x - a.position[0];
            let dy = y - a.position[1];
            let dz = z - a.position[2];
            sum += crate::physics::psi_squared(a.orbital.n, a.orbital.l, a.orbital.m, dx, dy, dz);
        }
        sum
    };

    #[cfg(not(target_arch = "wasm32"))]
    let raw: Vec<f64> = (0..total).into_par_iter().map(sample).collect();
    #[cfg(target_arch = "wasm32")]
    let raw: Vec<f64> = (0..total).map(sample).collect();

    let peak = raw.iter().copied().fold(0.0_f64, f64::max);
    let inv = if peak > 0.0 { 1.0 / peak } else { 0.0 };
    let data: Vec<f32> = raw.iter().map(|&v| (v * inv) as f32).collect();

    Volume { data, res, half_extent, peak }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Orbital, Scene};

    #[test]
    fn box_extent_grows_with_n_squared() {
        // Edge length = 6·n²·a₀ → half-edge = 3·n²·a₀
        assert!((box_extent(1) - 3.0).abs() < 1e-12);
        assert!((box_extent(2) - 12.0).abs() < 1e-12);
        assert!((box_extent(6) - 108.0).abs() < 1e-12);
    }

    /// The historical `bake(n, l, m, res)` implementation, reproduced inline
    /// for the regression test below. The acceptance criterion for the
    /// Scene-shaped bake API is that a single-hydrogen scene produces
    /// voxel-for-voxel identical output to this implementation.
    fn legacy_bake(n: u32, l: u32, m: i32, res: usize) -> Volume {
        let half_extent = box_extent(n);
        let step = 2.0 * half_extent / res as f64;
        let total = res * res * res;
        let sample = |idx: usize| -> f64 {
            let i = idx % res;
            let j = (idx / res) % res;
            let k = idx / (res * res);
            let x = -half_extent + (i as f64 + 0.5) * step;
            let y = -half_extent + (j as f64 + 0.5) * step;
            let z = -half_extent + (k as f64 + 0.5) * step;
            crate::physics::psi_squared(n, l, m, x, y, z)
        };
        let raw: Vec<f64> = (0..total).into_par_iter().map(sample).collect();
        let peak = raw.iter().copied().fold(0.0_f64, f64::max);
        let inv = if peak > 0.0 { 1.0 / peak } else { 0.0 };
        let data: Vec<f32> = raw.iter().map(|&v| (v * inv) as f32).collect();
        Volume { data, res, half_extent, peak }
    }

    #[test]
    fn bake_produces_unit_peak_after_normalization() {
        let v = bake_scene(&Scene::single_hydrogen(Orbital { n: 2, l: 1, m: 0 }), 32);
        let max = v.data.iter().copied().fold(0.0_f32, f32::max);
        assert!((max - 1.0).abs() < 1e-6, "expected peak 1.0, got {max}");
        assert!(v.peak > 0.0);
    }

    #[test]
    fn bake_integral_is_approximately_one() {
        // Re-derive the absolute density from `data * peak` and integrate.
        let v = bake_scene(&Scene::single_hydrogen(Orbital { n: 2, l: 1, m: 0 }), 64);
        let step = 2.0 * v.half_extent / v.res as f64;
        let dv = step.powi(3);
        let integral: f64 = v.data.iter().map(|&d| d as f64 * v.peak * dv).sum();
        assert!(
            (integral - 1.0).abs() < 0.10,
            "expected ~1.0, got {integral}"
        );
    }

    #[test]
    fn bake_scene_single_h_matches_legacy_bake() {
        // The Approach-3 refactor must be a pure plumbing change: a single
        // hydrogen atom at the origin must produce voxel-for-voxel identical
        // output to the pre-refactor bake. Identity (not approximate) is the
        // discipline this regression test enforces.
        let legacy = legacy_bake(2, 1, 0, 32);
        let scene = Scene::single_hydrogen(Orbital { n: 2, l: 1, m: 0 });
        let new = bake_scene(&scene, 32);

        assert_eq!(new.res, legacy.res);
        assert_eq!(new.half_extent, legacy.half_extent);
        assert_eq!(new.peak, legacy.peak);
        assert_eq!(new.data.len(), legacy.data.len());
        for (i, (a, b)) in new.data.iter().zip(legacy.data.iter()).enumerate() {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "voxel {i}: new={a} legacy={b}"
            );
        }
    }
}
