//! Adaptive cartesian box around the nucleus, sized to fit the current orbital.
//! Cubic box of edge length 6·n²·a₀; box_extent returns the half-edge (radius).

/// Half-edge of the cubic bounding box for orbital with principal quantum number n.
/// Returned in atomic units (a₀).
pub fn box_extent(n: u32) -> f64 {
    3.0 * (n as f64).powi(2)
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
}
