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
use crate::slater::z_eff;

/// Half-edge of the cubic bounding box for an orbital with principal quantum
/// number `n`. Returned in atomic units (a₀).
///
/// Bare-hydrogen sizing (`z_eff = 1`): `3·n²·a₀`. For an arbitrary
/// effective charge, the orbital shrinks as `1/z_eff` (the natural length
/// scale of a hydrogen-like atom is `n²·a₀/z_eff`), so the box scales the
/// same way.
pub fn box_extent(n: u32) -> f64 {
    box_extent_zeff(n, 1.0)
}

/// Half-edge for an orbital with principal `n` and effective nuclear
/// charge `z_eff`. `box_extent(n) == box_extent_zeff(n, 1.0)`.
pub fn box_extent_zeff(n: u32, z_eff: f64) -> f64 {
    let z = z_eff.max(0.1);
    3.0 * (n as f64).powi(2) / z
}

/// Per-atom orbital "radius" used by the adaptive box sizing. Scales with
/// `n²` and inversely with the resolved effective charge so high-Z
/// orbitals tighten in.
fn atom_radius(orbital: Orbital, z_eff: f64) -> f64 {
    box_extent_zeff(orbital.n, z_eff)
}

/// Resolve the effective nuclear charge for an atom under the scene's view
/// settings. When `use_bare_z` is true the override returns the element's
/// bare atomic number `Z` (no Slater shielding); otherwise it delegates to
/// `slater::z_eff`.
fn resolve_z_eff(atom: &Atom, use_bare_z: bool) -> f64 {
    if use_bare_z {
        // Bare atomic number. Falls back to 1.0 for unknown elements so
        // the bake never produces NaN / a zero-size box.
        let z = atom.element.0.max(1) as f64;
        return z;
    }
    z_eff(atom.element, atom.orbital.n, atom.orbital.l)
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
/// Per atom, `z_eff` is resolved from the element + orbital via Slater's
/// rules, unless `scene.view.use_bare_z` is set — in which case the bare
/// atomic number `Z` is used instead.
pub fn bake_scene(scene: &Scene, res: usize) -> Volume {
    let use_bare_z = scene.view.use_bare_z;

    // Precompute per-atom z_eff so the (parallel) sample closure doesn't
    // redo Slater lookups for every voxel.
    let z_effs: Vec<f64> = scene
        .atoms
        .iter()
        .map(|a| resolve_z_eff(a, use_bare_z))
        .collect();

    let half_extent = scene
        .atoms
        .iter()
        .zip(z_effs.iter())
        .map(|(a, &z)| {
            let r = (a.position[0] * a.position[0]
                + a.position[1] * a.position[1]
                + a.position[2] * a.position[2])
                .sqrt();
            r + atom_radius(a.orbital, z)
        })
        .fold(0.0_f64, f64::max);
    let step = 2.0 * half_extent / res as f64;
    let total = res * res * res;

    let atoms: &[Atom] = &scene.atoms;
    let z_effs_slice: &[f64] = &z_effs;
    let sample = |idx: usize| -> f64 {
        let i = idx % res;
        let j = (idx / res) % res;
        let k = idx / (res * res);
        let x = -half_extent + (i as f64 + 0.5) * step;
        let y = -half_extent + (j as f64 + 0.5) * step;
        let z = -half_extent + (k as f64 + 0.5) * step;
        let mut sum = 0.0_f64;
        for (a, &ze) in atoms.iter().zip(z_effs_slice.iter()) {
            let dx = x - a.position[0];
            let dy = y - a.position[1];
            let dz = z - a.position[2];
            sum +=
                crate::physics::psi_squared(a.orbital.n, a.orbital.l, a.orbital.m, ze, dx, dy, dz);
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
            // Bare-hydrogen z_eff=1.0 — what the pre-Slater bake assumed.
            crate::physics::psi_squared(n, l, m, 1.0, x, y, z)
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
    fn bake_scene_carbon_2p_smaller_than_hydrogen_2p() {
        use crate::scene::{Atom, ElementId, View};
        // The user-visible payoff: carbon's 2p has z_eff = 3.25, so its
        // bounding box must shrink relative to hydrogen's 2p (z_eff = 1).
        let h_scene = Scene::single_hydrogen(Orbital { n: 2, l: 1, m: 0 });
        let c_scene = Scene {
            atoms: vec![Atom {
                element: ElementId(6), // Carbon
                position: [0.0, 0.0, 0.0],
                orbital: Orbital { n: 2, l: 1, m: 0 },
            }],
            view: View::default(),
        };
        let h = bake_scene(&h_scene, 16);
        let c = bake_scene(&c_scene, 16);
        assert!(
            c.half_extent < h.half_extent,
            "carbon box {} should be smaller than hydrogen box {}",
            c.half_extent,
            h.half_extent,
        );
    }

    #[test]
    fn bake_scene_bare_z_overrides_slater() {
        use crate::scene::{Atom, ElementId, View};
        // Two carbons, identical orbital — one with use_bare_z=false
        // (Slater z_eff=3.25) and one with use_bare_z=true (bare Z=6).
        // The bare-Z bake must produce an even tighter box.
        let orbital = Orbital { n: 2, l: 1, m: 0 };
        let atom = Atom { element: ElementId(6), position: [0.0, 0.0, 0.0], orbital };
        let slater_scene = Scene { atoms: vec![atom], view: View::default() };
        let bare_scene = Scene {
            atoms: vec![atom],
            view: View { use_bare_z: true, ..View::default() },
        };
        let slater = bake_scene(&slater_scene, 16);
        let bare = bake_scene(&bare_scene, 16);
        assert!(
            bare.half_extent < slater.half_extent,
            "bare-Z box {} should be tighter than Slater box {}",
            bare.half_extent,
            slater.half_extent,
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
