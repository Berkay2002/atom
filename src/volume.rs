//! Adaptive cartesian box around the nucleus, sized to fit the current orbital.
//! Cubic box of edge length 6·n²·a₀; box_extent returns the half-edge (radius).
//! `bake` evaluates |ψ|² on a regular grid (rayon-parallelized) and peak-normalizes.

use rayon::prelude::*;

/// Half-edge of the cubic bounding box for orbital with principal quantum number n.
/// Returned in atomic units (a₀).
pub fn box_extent(n: u32) -> f64 {
    3.0 * (n as f64).powi(2)
}

/// Baked volume: peak-normalized density on a cubic grid centered at the nucleus.
pub struct Volume {
    pub data: Vec<f32>,   // length = res³, row-major, x fastest then y then z
    pub res: usize,
    pub half_extent: f64, // a₀
    pub peak: f64,        // absolute peak |ψ|² before normalization (for HUD)
}

/// Bake the volume for orbital (n, l, m) at the given grid resolution.
/// Density is sampled at voxel centers, then divided by the in-grid peak so
/// `data` lies in [0, 1]. The absolute peak is preserved in `peak`.
pub fn bake(n: u32, l: u32, m: i32, res: usize) -> Volume {
    let half_extent = box_extent(n);
    let step = 2.0 * half_extent / res as f64;
    let total = res * res * res;

    let raw: Vec<f64> = (0..total)
        .into_par_iter()
        .map(|idx| {
            let i = idx % res;
            let j = (idx / res) % res;
            let k = idx / (res * res);
            let x = -half_extent + (i as f64 + 0.5) * step;
            let y = -half_extent + (j as f64 + 0.5) * step;
            let z = -half_extent + (k as f64 + 0.5) * step;
            crate::physics::psi_squared(n, l, m, x, y, z)
        })
        .collect();

    let peak = raw.iter().copied().fold(0.0_f64, f64::max);
    let inv = if peak > 0.0 { 1.0 / peak } else { 0.0 };
    let data: Vec<f32> = raw.iter().map(|&v| (v * inv) as f32).collect();

    Volume { data, res, half_extent, peak }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_extent_grows_with_n_squared() {
        // Edge length = 6·n²·a₀ → half-edge = 3·n²·a₀
        assert!((box_extent(1) - 3.0).abs() < 1e-12);
        assert!((box_extent(2) - 12.0).abs() < 1e-12);
        assert!((box_extent(6) - 108.0).abs() < 1e-12);
    }

    #[test]
    fn bake_produces_unit_peak_after_normalization() {
        let v = bake(2, 1, 0, 32);
        let max = v.data.iter().copied().fold(0.0_f32, f32::max);
        assert!((max - 1.0).abs() < 1e-6, "expected peak 1.0, got {max}");
        assert!(v.peak > 0.0);
    }

    #[test]
    fn bake_integral_is_approximately_one() {
        // Re-derive the absolute density from `data * peak` and integrate.
        let v = bake(2, 1, 0, 64);
        let step = 2.0 * v.half_extent / v.res as f64;
        let dv = step.powi(3);
        let integral: f64 = v.data.iter().map(|&d| d as f64 * v.peak * dv).sum();
        assert!(
            (integral - 1.0).abs() < 0.10,
            "expected ~1.0, got {integral}"
        );
    }
}
