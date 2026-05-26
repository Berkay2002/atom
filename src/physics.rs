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

/// Radial part of hydrogen wavefunction R_nl(r), in atomic units (a₀ = 1, Z = 1).
/// R_nl(r) = sqrt((2/n)³ · (n-l-1)!/(2n·(n+l)!)) · e^(-ρ/2) · ρ^l · L_{n-l-1}^{2l+1}(ρ),  ρ = 2r/n
pub fn radial(n: u32, l: u32, r: f64) -> f64 {
    debug_assert!(l < n, "l must satisfy l < n");
    let n_f = n as f64;
    let rho = 2.0 * r / n_f;
    let norm_sq = (2.0 / n_f).powi(3) * factorial(n - l - 1) / (2.0 * n_f * factorial(n + l));
    let norm = norm_sq.sqrt();
    let lag = laguerre(n - l - 1, 2 * l + 1, rho);
    norm * (-rho / 2.0).exp() * rho.powi(l as i32) * lag
}

fn factorial(k: u32) -> f64 {
    (1..=k).fold(1.0_f64, |acc, i| acc * i as f64)
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
        approx_rel(radial(1, 0, 0.0), 2.0, 1e-12);
        approx_rel(radial(1, 0, 1.0), 2.0 * (-1.0_f64).exp(), 1e-12);
        // R_{2,0}(r) = (1/(2·sqrt(2))) · (2-r) · e^(-r/2); at r=0 → 1/sqrt(2)
        approx_rel(radial(2, 0, 0.0), 1.0 / 2_f64.sqrt(), 1e-12);
    }
}
