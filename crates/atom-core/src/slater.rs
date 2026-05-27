//! Slater's-rules effective nuclear charge `z_eff(element, n, l)`.
//!
//! Standard formulation (Slater 1930). Orbitals are grouped:
//!
//!   (1s) | (2s, 2p) | (3s, 3p) | (3d) | (4s, 4p) | (4d) | (4f) | ...
//!
//! For the electron whose `z_eff` we're computing, in group `N`:
//!
//!   * 0 from electrons in groups *above* `N` (further from nucleus).
//!   * Same-group contribution: 0.35 per *other* electron in `N` (or 0.30
//!     when `N` is `(1s)`).
//!   * If the target electron is s or p: 0.85 per electron in the `(n-1)`
//!     shell groups, 1.00 per electron in deeper groups.
//!   * If the target electron is d or f: 1.00 per electron in *every*
//!     lower group (no 0.85 reduction).
//!
//! `z_eff = Z - sigma`.
//!
//! For elements H–Ar this only ever touches the s/p branch — d/f are not
//! occupied. The d/f rules are implemented anyway so an unoccupied 3d or
//! 4f orbital on, say, Carbon still returns a sensible value.
//!
//! ## Unoccupied-orbital convention
//!
//! If `(n, l)` is *not* in the element's ground-state config (e.g. asking
//! for Carbon's 4f), we compute screening as if a single test electron
//! were placed in that orbital — the existing electrons screen it
//! normally, and same-group co-residents add their 0.35 (or 0.30) if any
//! happen to share the group. The "self" 0.35 does NOT apply: the test
//! electron is the one being computed for, not a screener.

use crate::element::element_data;
use crate::scene::ElementId;

/// Internal grouping key for Slater's rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Group {
    /// `(1s)` — uses 0.30 same-group contribution instead of 0.35.
    OneS,
    /// `(ns, np)` for n >= 2.
    SP(u32),
    /// `(nd)` for n >= 3.
    D(u32),
    /// `(nf)` for n >= 4.
    F(u32),
}

/// Map a `(n, l)` pair to its Slater group.
fn group_of(n: u32, l: u32) -> Group {
    match l {
        0 if n == 1 => Group::OneS,
        0 | 1 => Group::SP(n),
        2 => Group::D(n),
        _ => Group::F(n),
    }
}

/// Effective `n` for a group, for ordering by shell distance.
fn group_n(g: Group) -> u32 {
    match g {
        Group::OneS => 1,
        Group::SP(n) | Group::D(n) | Group::F(n) => n,
    }
}

/// Effective nuclear charge `Z_eff` for an electron in orbital `(n, l)`
/// of `element`, computed via Slater's rules.
///
/// **Convention for orbitals not occupied in the ground state:** a single
/// test electron is placed in that orbital and screened by the existing
/// configuration. Same-group co-residents (if any in the element's actual
/// config) still contribute their 0.35 / 0.30; the test electron itself
/// does *not* self-screen.
pub fn z_eff(element: ElementId, n: u32, l: u32) -> f64 {
    let Some(data) = element_data(element) else {
        return 1.0;
    };
    let z = data.atomic_number as f64;

    // Single-electron atom (only Hydrogen in our 1..=18 set). The lone
    // electron *is* the test electron regardless of orbital, so there is
    // nothing to screen with. This keeps `single_hydrogen(any_orbital)`
    // identical to the pre-Slater bake (z_eff = Z = 1.0). Naively applying
    // the "unoccupied" convention would let the 1s electron screen a 2p
    // test electron — physically wrong for a one-electron atom.
    if data.atomic_number == 1 {
        return z;
    }

    let target_group = group_of(n, l);
    let target_n = group_n(target_group);
    let target_is_d_or_f = matches!(target_group, Group::D(_) | Group::F(_));

    // Is the target (n, l) actually one of the entries in the config?
    // If yes, the entry holding it contributes (count - 1) same-group
    // screeners (the target itself doesn't self-screen). If no — the
    // unoccupied-orbital convention — any same-group residents from
    // other entries still screen the test electron at the normal rate.
    let target_is_occupied = data
        .electron_config
        .iter()
        .any(|(en, el, _)| *en == n && *el == l);

    let mut sigma = 0.0_f64;

    for &(en, el, count) in data.electron_config.iter() {
        let g = group_of(en, el);
        let g_n = group_n(g);

        if g == target_group {
            // Same group. If the orbital is occupied, one of the
            // electrons in this entry *is* the target electron we're
            // computing for and must not self-screen — subtract one.
            // (If multiple entries map to the same group, the "self"
            // only belongs to whichever entry holds (n, l).)
            let screeners = if target_is_occupied && en == n && el == l {
                count.saturating_sub(1)
            } else {
                count
            };
            let per = if target_group == Group::OneS { 0.30 } else { 0.35 };
            sigma += screeners as f64 * per;
        } else if g_n < target_n {
            // Deeper group.
            if target_is_d_or_f {
                // d/f targets see every deeper electron at full 1.00.
                sigma += count as f64 * 1.00;
            } else if g_n + 1 == target_n {
                // One shell lower: 0.85 for s/p targets.
                sigma += count as f64 * 0.85;
            } else {
                // Two or more shells lower: 1.00.
                sigma += count as f64 * 1.00;
            }
        }
        // g_n > target_n (outer shells): contribute 0.
        // g_n == target_n but different group (e.g. 3p target vs 3d group):
        // by Slater's grouping rule, 3d is "above" 3s/3p (a separate group
        // higher in the build-up order); contribution 0. We hit this branch
        // because `g != target_group` and `g_n == target_n`, so neither of
        // the `g_n < target_n` arms fires — sigma unchanged.
    }

    (z - sigma).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "expected {b}, got {a}");
    }

    #[test]
    fn hydrogen_1s() {
        // Z=1, no other electrons → sigma=0, z_eff=1.0.
        close(z_eff(ElementId(1), 1, 0), 1.0, 1e-9);
    }

    #[test]
    fn helium_1s() {
        // Two 1s electrons; one other contributes 0.30 → z_eff = 2 - 0.30 = 1.70.
        close(z_eff(ElementId(2), 1, 0), 1.70, 1e-9);
    }

    #[test]
    fn carbon_2p() {
        // C: 1s² 2s² 2p². For a 2p electron:
        //   same-group (2s, 2p): 3 other electrons × 0.35 = 1.05
        //   (n-1)=1 shell: 2 × 0.85 = 1.70
        //   sigma = 2.75 → z_eff = 6 - 2.75 = 3.25
        close(z_eff(ElementId(6), 2, 1), 3.25, 1e-9);
    }

    #[test]
    fn carbon_2s() {
        // Same group as 2p by Slater grouping, so identical screening → 3.25.
        close(z_eff(ElementId(6), 2, 0), 3.25, 1e-9);
    }

    #[test]
    fn fluorine_2p() {
        // F: 1s² 2s² 2p⁵. For a 2p electron:
        //   same-group: 6 other × 0.35 = 2.10
        //   (n-1): 2 × 0.85 = 1.70
        //   sigma = 3.80 → z_eff = 9 - 3.80 = 5.20
        close(z_eff(ElementId(9), 2, 1), 5.20, 1e-9);
    }

    #[test]
    fn sodium_3s() {
        // Na: 1s² 2s² 2p⁶ 3s¹. For the 3s electron:
        //   same-group: 0 other × 0.35 = 0
        //   (n-1) shell (2s,2p): 8 × 0.85 = 6.80
        //   (n-2) shell (1s):    2 × 1.00 = 2.00
        //   sigma = 8.80 → z_eff = 11 - 8.80 = 2.20
        close(z_eff(ElementId(11), 3, 0), 2.20, 1e-9);
    }

    #[test]
    fn lithium_2s() {
        // Li: 1s² 2s¹. 2s electron: same-group 0, (n-1)=1: 2 × 0.85 = 1.70.
        // sigma = 1.70 → z_eff = 3 - 1.70 = 1.30.
        close(z_eff(ElementId(3), 2, 0), 1.30, 1e-9);
    }

    #[test]
    fn unoccupied_orbital_carbon_3s() {
        // Carbon's 3s is not in the ground-state config. Convention: place
        // a test electron there. Carbon = 1s² 2s² 2p².
        // For 3s: same-group: 0; (n-1)=2 shell: 4 × 0.85 = 3.40;
        // (n-2)=1 shell: 2 × 1.00 = 2.00; sigma = 5.40; z_eff = 0.60.
        close(z_eff(ElementId(6), 3, 0), 0.60, 1e-9);
    }

    #[test]
    fn unknown_element_falls_back_to_hydrogen() {
        // Any out-of-range ElementId yields z_eff=1.0 (bare hydrogen).
        close(z_eff(ElementId(0), 1, 0), 1.0, 1e-9);
        close(z_eff(ElementId(99), 2, 1), 1.0, 1e-9);
    }
}
