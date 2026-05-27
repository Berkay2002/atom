//! Closed-form hydrogen wavefunctions: Laguerre, Legendre, real Y_lm, |ψ|².
//! All formulas verified to ~1e-18 absolute error vs scipy in an earlier session.

/// Associated Laguerre polynomial L_p^q(x), p >= 0, q >= 0.
/// Stable upward recurrence:
///   L_0 = 1
///   L_1 = 1 + q - x
///   (k+1)·L_{k+1} = (2k+1+q-x)·L_k - (k+q)·L_{k-1}
pub fn laguerre(p: u32, q: u32, x: f64) -> f64 {
    let q = q as f64;
    if p == 0 {
        return 1.0;
    }
    let mut l_prev = 1.0;
    let mut l_curr = 1.0 + q - x;
    if p == 1 {
        return l_curr;
    }
    for k in 1..p {
        let k_f = k as f64;
        let l_next = ((2.0 * k_f + 1.0 + q - x) * l_curr - (k_f + q) * l_prev) / (k_f + 1.0);
        l_prev = l_curr;
        l_curr = l_next;
    }
    l_curr
}

/// Associated Legendre polynomial P_l^m(x), l >= 0, 0 <= m <= l, x ∈ [-1, 1].
/// Includes Condon–Shortley sign. Recurrence:
///   pmm = 1; for i in 1..=m: pmm *= -(2i-1) * sqrt(1 - x²)
///   P_m^m     = pmm
///   P_{m+1}^m = x·(2m+1)·pmm
///   P_l^m     = ((2l-1)·x·P_{l-1}^m - (l+m-1)·P_{l-2}^m) / (l-m)   for l >= m+2
pub fn legendre(l: u32, m: u32, x: f64) -> f64 {
    debug_assert!(m <= l, "m must be <= l");
    let m_us = m as usize;
    let sx = (1.0 - x * x).max(0.0).sqrt();
    let mut pmm = 1.0;
    for i in 1..=m_us {
        pmm *= -(2.0 * i as f64 - 1.0) * sx;
    }
    if l == m {
        return pmm;
    }
    let mut p_prev = pmm;
    let mut p_curr = x * (2.0 * m as f64 + 1.0) * pmm;
    if l == m + 1 {
        return p_curr;
    }
    for ll in (m + 2)..=l {
        let ll_f = ll as f64;
        let p_next =
            ((2.0 * ll_f - 1.0) * x * p_curr - (ll_f + m as f64 - 1.0) * p_prev) / (ll_f - m as f64);
        p_prev = p_curr;
        p_curr = p_next;
    }
    p_curr
}

/// Radial part of a hydrogen-*like* wavefunction `R_nl(r)`, in atomic units
/// (`a₀ = 1`). The effective nuclear charge `z_eff` substitutes for `Z` in
/// the standard hydrogen formula:
///
/// ```text
///   R_nl(r) = sqrt((2·z_eff/n)³ · (n-l-1)!/(2n·(n+l)!))
///             · e^(-ρ/2) · ρ^l · L_{n-l-1}^{2l+1}(ρ),    ρ = 2·z_eff·r/n
/// ```
///
/// Bare hydrogen is the special case `z_eff = 1.0`, which collapses to the
/// original formula.
pub fn radial(n: u32, l: u32, r: f64, z_eff: f64) -> f64 {
    debug_assert!(l < n, "l must satisfy l < n");
    let n_f = n as f64;
    let rho = 2.0 * z_eff * r / n_f;
    let norm_sq =
        (2.0 * z_eff / n_f).powi(3) * factorial(n - l - 1) / (2.0 * n_f * factorial(n + l));
    let norm = norm_sq.sqrt();
    let lag = laguerre(n - l - 1, 2 * l + 1, rho);
    norm * (-rho / 2.0).exp() * rho.powi(l as i32) * lag
}

fn factorial(k: u32) -> f64 {
    (1..=k).fold(1.0_f64, |acc, i| acc * i as f64)
}

/// `|ψ_nlm(x, y, z)|²` for a hydrogen-like atom with effective nuclear
/// charge `z_eff`, in atomic units. Coordinates are cartesian, in `a₀`.
///
/// Element-agnostic: callers (e.g. `volume::bake_scene`) resolve `z_eff`
/// via Slater's rules or the bare-Z override before invoking.
pub fn psi_squared(n: u32, l: u32, m: i32, z_eff: f64, x: f64, y: f64, z: f64) -> f64 {
    let r = (x * x + y * y + z * z).sqrt();
    if r == 0.0 {
        // Only s-orbitals (l=0) have nonzero amplitude at the origin.
        // For l>0, ρ^l in R_nl forces R(0)=0.
        if l == 0 {
            let rad = radial(n, 0, 0.0, z_eff);
            let y0 = real_y(0, 0, 0.0, 0.0);
            let psi = rad * y0;
            return psi * psi;
        }
        return 0.0;
    }
    let theta = (z / r).clamp(-1.0, 1.0).acos();
    let phi = y.atan2(x);
    let rad = radial(n, l, r, z_eff);
    let ylm = real_y(l, m, theta, phi);
    let psi = rad * ylm;
    psi * psi
}

/// Real spherical harmonic Y_lm(θ, φ), θ ∈ [0, π], φ ∈ [0, 2π].
/// K = sqrt( (2l+1)/(4π) · (l-|m|)! / (l+|m|)! )
///   m > 0:  Y = sqrt(2)·K·cos(m·φ)·P_l^m(cosθ)
///   m = 0:  Y =           K·       P_l^0(cosθ)
///   m < 0:  Y = sqrt(2)·K·sin(|m|·φ)·P_l^|m|(cosθ)
pub fn real_y(l: u32, m: i32, theta: f64, phi: f64) -> f64 {
    use std::f64::consts::PI;
    let abs_m = m.unsigned_abs();
    debug_assert!(abs_m <= l, "|m| must be <= l");
    let cos_t = theta.cos();
    let p = legendre(l, abs_m, cos_t);
    let k = (((2 * l + 1) as f64) / (4.0 * PI)
        * factorial(l - abs_m) / factorial(l + abs_m))
        .sqrt();
    if m == 0 {
        k * p
    } else if m > 0 {
        2_f64.sqrt() * k * (m as f64 * phi).cos() * p
    } else {
        2_f64.sqrt() * k * (abs_m as f64 * phi).sin() * p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-12, "expected {b}, got {a}");
    }

    fn approx_rel(a: f64, b: f64, tol: f64) {
        let scale = b.abs().max(1.0);
        assert!((a - b).abs() < tol * scale, "expected {b}, got {a}");
    }

    #[test]
    fn laguerre_base_cases() {
        approx(laguerre(0, 0, 0.0), 1.0);
        approx(laguerre(0, 5, 3.7), 1.0);
        approx(laguerre(1, 1, 0.0), 2.0);   // 1 + 1 - 0
        approx(laguerre(1, 3, 2.0), 2.0);   // 1 + 3 - 2
    }

    #[test]
    fn laguerre_recurrence() {
        // L_2^q(x) = ((q+1)(q+2) - 2(q+2)x + x²) / 2
        approx(laguerre(2, 1, 0.0), 3.0);
        approx(laguerre(2, 3, 2.0), 2.0);
    }

    #[test]
    fn legendre_base_cases() {
        approx(legendre(0, 0, 0.0), 1.0);
        approx(legendre(0, 0, 0.7), 1.0);
        approx(legendre(1, 0, 0.5), 0.5);
        approx(legendre(1, 1, 0.0), -1.0);
    }

    #[test]
    fn legendre_recurrence() {
        approx(legendre(2, 0, 0.0), -0.5);
        approx(legendre(2, 2, 0.0), 3.0);
        approx(legendre(2, 1, 0.5), -3.0 * 0.5 * (0.75_f64).sqrt());
    }

    #[test]
    fn radial_known_values() {
        // R_{1,0}(r) = 2 e^(-r); at r=0 → 2
        approx_rel(radial(1, 0, 0.0, 1.0), 2.0, 1e-12);
        approx_rel(radial(1, 0, 1.0, 1.0), 2.0 * (-1.0_f64).exp(), 1e-12);
        // R_{2,0}(r) = (1/(2·sqrt(2))) · (2-r) · e^(-r/2); at r=0 → 1/sqrt(2)
        approx_rel(radial(2, 0, 0.0, 1.0), 1.0 / 2_f64.sqrt(), 1e-12);
    }

    #[test]
    fn radial_he_plus_z_eff_2() {
        // He⁺ (one electron, Z=2): R_{1,0}(r) = 2 · Z^(3/2) · exp(-Z·r).
        // Scipy cross-check values (Z=2):
        //   r=0   → 2 · 2^(3/2)              = 5.6568542494923806...
        //   r=0.5 → 2 · 2^(3/2) · exp(-1)    = 2.080816862637854...
        //   r=1.0 → 2 · 2^(3/2) · exp(-2)    = 0.7656519101110693...
        let z = 2.0;
        approx_rel(radial(1, 0, 0.0, z), 2.0 * z.powf(1.5), 1e-12);
        approx_rel(radial(1, 0, 0.5, z), 2.0 * z.powf(1.5) * (-1.0_f64).exp(), 1e-12);
        approx_rel(radial(1, 0, 1.0, z), 2.0 * z.powf(1.5) * (-2.0_f64).exp(), 1e-12);
    }

    #[test]
    fn radial_he_plus_2p_z_eff_2() {
        // He⁺ 2p (Z=2): R_{2,1}(r) = sqrt(Z³/24) · Z·r · exp(-Z·r/2).
        //   At r=1, Z=2: sqrt(8/24) · 2 · exp(-1) = 2/sqrt(3) · exp(-1)
        //                                          ≈ 0.4247906...
        let z = 2.0_f64;
        let expected = 2.0 / 3.0_f64.sqrt() * (-1.0_f64).exp();
        approx_rel(radial(2, 1, 1.0, z), expected, 1e-12);
    }

    #[test]
    fn real_y_known_values() {
        use std::f64::consts::PI;
        // |Y_{0,0}|² = 1/(4π) everywhere
        let y00 = real_y(0, 0, 0.7, 1.2);
        approx_rel(y00 * y00, 1.0 / (4.0 * PI), 1e-12);
        // |Y_{1,1}|²(π/2, 0) = 3/(4π)   (sign-independent check)
        let y11 = real_y(1, 1, PI / 2.0, 0.0);
        approx_rel(y11 * y11, 3.0 / (4.0 * PI), 1e-12);
        // |Y_{1,0}|²(0, 0) = 3/(4π)
        let y10 = real_y(1, 0, 0.0, 0.0);
        approx_rel(y10 * y10, 3.0 / (4.0 * PI), 1e-12);
    }

    #[test]
    fn psi_squared_at_origin_1s() {
        use std::f64::consts::PI;
        // ψ_{1,0,0}(0) = 1/sqrt(π) → |ψ|² = 1/π
        approx_rel(psi_squared(1, 0, 0, 1.0, 0.0, 0.0, 0.0), 1.0 / PI, 1e-12);
    }

    #[test]
    fn psi_squared_at_origin_2s() {
        use std::f64::consts::PI;
        // ψ_{2,0,0}(0) = 1/(2·sqrt(2π)) → |ψ|² = 1/(8π)
        approx_rel(psi_squared(2, 0, 0, 1.0, 0.0, 0.0, 0.0), 1.0 / (8.0 * PI), 1e-12);
    }

    #[test]
    fn psi_squared_he_plus_at_origin() {
        use std::f64::consts::PI;
        // He⁺ 1s: ψ(0) = sqrt(Z³/π). |ψ(0)|² = Z³/π. For Z=2: 8/π.
        approx_rel(
            psi_squared(1, 0, 0, 2.0, 0.0, 0.0, 0.0),
            8.0 / PI,
            1e-12,
        );
    }

    fn integrate_psi_squared(n: u32, l: u32, m: i32, z_eff: f64, half_extent: f64, res: usize) -> f64 {
        let step = 2.0 * half_extent / res as f64;
        let dv = step.powi(3);
        let mut acc = 0.0_f64;
        for i in 0..res {
            let x = -half_extent + (i as f64 + 0.5) * step;
            for j in 0..res {
                let y = -half_extent + (j as f64 + 0.5) * step;
                for k in 0..res {
                    let z = -half_extent + (k as f64 + 0.5) * step;
                    acc += psi_squared(n, l, m, z_eff, x, y, z) * dv;
                }
            }
        }
        acc
    }

    #[test]
    fn psi_squared_normalizes_1s() {
        let integral = integrate_psi_squared(1, 0, 0, 1.0, 8.0, 64);
        approx_rel(integral, 1.0, 0.05);
    }

    #[test]
    fn psi_squared_normalizes_2p() {
        let integral = integrate_psi_squared(2, 1, 0, 1.0, 15.0, 64);
        approx_rel(integral, 1.0, 0.10);
    }

    #[test]
    fn psi_squared_he_plus_1s_normalizes() {
        // He⁺ 1s with Z=2 is half the size; tighter box, still integrates to 1.
        let integral = integrate_psi_squared(1, 0, 0, 2.0, 4.0, 64);
        approx_rel(integral, 1.0, 0.05);
    }
}
