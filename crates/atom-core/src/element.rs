//! Static table of element data for the first 18 elements (H–Ar).
//!
//! Each entry carries the symbol, name, atomic number, ground-state
//! electron configuration (machine-readable), and a human-readable config
//! string for captions. Lookup is by `ElementId` (atomic number).
//!
//! The electron configuration is stored as a slice of `(n, l, count)`
//! triples — `slater.rs` consumes this directly when resolving the
//! per-orbital screening sum.

use crate::scene::ElementId;

/// A single `(n, l, count)` entry in an electron configuration.
///
/// `n` is the principal quantum number, `l` is the orbital angular
/// momentum (0=s, 1=p, 2=d, 3=f), `count` is the number of electrons
/// occupying that subshell in the element's ground state.
pub type ConfigEntry = (u32, u32, u32);

/// Static data for one element.
#[derive(Debug, Clone, Copy)]
pub struct ElementData {
    /// Chemical symbol, e.g. "H", "He", "Li".
    pub symbol: &'static str,
    /// Full English name, e.g. "Hydrogen".
    pub name: &'static str,
    /// Atomic number `Z` (= number of protons = number of electrons in the
    /// neutral atom).
    pub atomic_number: u32,
    /// Ground-state electron configuration as a list of `(n, l, count)`.
    pub electron_config: &'static [ConfigEntry],
    /// Human-readable configuration string for captions, e.g.
    /// `"1s² 2s² 2p²"` for Carbon. Uses Unicode superscripts.
    pub config_str: &'static str,
}

// Ground-state configurations for H–Ar. Build-up order is Madelung-correct
// for these 18 elements (the 4s/3d crossover doesn't appear until K/Ca,
// which we exclude).
const C_H:  &[ConfigEntry] = &[(1, 0, 1)];
const C_HE: &[ConfigEntry] = &[(1, 0, 2)];
const C_LI: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 1)];
const C_BE: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2)];
const C_B:  &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 1)];
const C_C:  &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 2)];
const C_N:  &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 3)];
const C_O:  &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 4)];
const C_F:  &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 5)];
const C_NE: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6)];
const C_NA: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 1)];
const C_MG: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2)];
const C_AL: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 1)];
const C_SI: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 2)];
const C_P:  &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 3)];
const C_S:  &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 4)];
const C_CL: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 5)];
const C_AR: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 6)];

/// Table of all 18 supported elements (H through Ar), in atomic-number
/// order. Index `i` holds element with atomic number `i + 1`.
pub const ELEMENTS: [ElementData; 18] = [
    ElementData { symbol: "H",  name: "Hydrogen",   atomic_number: 1,  electron_config: C_H,  config_str: "1s\u{00B9}" },
    ElementData { symbol: "He", name: "Helium",     atomic_number: 2,  electron_config: C_HE, config_str: "1s\u{00B2}" },
    ElementData { symbol: "Li", name: "Lithium",    atomic_number: 3,  electron_config: C_LI, config_str: "1s\u{00B2} 2s\u{00B9}" },
    ElementData { symbol: "Be", name: "Beryllium",  atomic_number: 4,  electron_config: C_BE, config_str: "1s\u{00B2} 2s\u{00B2}" },
    ElementData { symbol: "B",  name: "Boron",      atomic_number: 5,  electron_config: C_B,  config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{00B9}" },
    ElementData { symbol: "C",  name: "Carbon",     atomic_number: 6,  electron_config: C_C,  config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{00B2}" },
    ElementData { symbol: "N",  name: "Nitrogen",   atomic_number: 7,  electron_config: C_N,  config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{00B3}" },
    ElementData { symbol: "O",  name: "Oxygen",     atomic_number: 8,  electron_config: C_O,  config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{2074}" },
    ElementData { symbol: "F",  name: "Fluorine",   atomic_number: 9,  electron_config: C_F,  config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{2075}" },
    ElementData { symbol: "Ne", name: "Neon",       atomic_number: 10, electron_config: C_NE, config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{2076}" },
    ElementData { symbol: "Na", name: "Sodium",     atomic_number: 11, electron_config: C_NA, config_str: "[Ne] 3s\u{00B9}" },
    ElementData { symbol: "Mg", name: "Magnesium",  atomic_number: 12, electron_config: C_MG, config_str: "[Ne] 3s\u{00B2}" },
    ElementData { symbol: "Al", name: "Aluminium",  atomic_number: 13, electron_config: C_AL, config_str: "[Ne] 3s\u{00B2} 3p\u{00B9}" },
    ElementData { symbol: "Si", name: "Silicon",    atomic_number: 14, electron_config: C_SI, config_str: "[Ne] 3s\u{00B2} 3p\u{00B2}" },
    ElementData { symbol: "P",  name: "Phosphorus", atomic_number: 15, electron_config: C_P,  config_str: "[Ne] 3s\u{00B2} 3p\u{00B3}" },
    ElementData { symbol: "S",  name: "Sulfur",     atomic_number: 16, electron_config: C_S,  config_str: "[Ne] 3s\u{00B2} 3p\u{2074}" },
    ElementData { symbol: "Cl", name: "Chlorine",   atomic_number: 17, electron_config: C_CL, config_str: "[Ne] 3s\u{00B2} 3p\u{2075}" },
    ElementData { symbol: "Ar", name: "Argon",      atomic_number: 18, electron_config: C_AR, config_str: "[Ne] 3s\u{00B2} 3p\u{2076}" },
];

/// Look up an element by its `ElementId` (atomic number). Returns `None`
/// for any atomic number outside `1..=18`.
pub fn element_data(id: ElementId) -> Option<&'static ElementData> {
    let z = id.0;
    if z == 0 || z as usize > ELEMENTS.len() {
        return None;
    }
    Some(&ELEMENTS[(z - 1) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn has_eighteen_entries() {
        assert_eq!(ELEMENTS.len(), 18);
    }

    #[test]
    fn atomic_numbers_are_one_through_eighteen_contiguous() {
        for (i, e) in ELEMENTS.iter().enumerate() {
            assert_eq!(e.atomic_number, (i + 1) as u32, "row {i} mismatched");
        }
    }

    #[test]
    fn no_duplicate_symbols() {
        let mut seen: HashSet<&str> = HashSet::new();
        for e in ELEMENTS.iter() {
            assert!(seen.insert(e.symbol), "duplicate symbol {}", e.symbol);
        }
        assert_eq!(seen.len(), 18);
    }

    #[test]
    fn electron_counts_sum_to_atomic_number() {
        // Neutral-atom ground state: total electrons = Z.
        for e in ELEMENTS.iter() {
            let total: u32 = e.electron_config.iter().map(|(_, _, c)| c).sum();
            assert_eq!(total, e.atomic_number, "{} electron count mismatch", e.symbol);
        }
    }

    #[test]
    fn lookup_by_id() {
        assert_eq!(element_data(ElementId(1)).unwrap().symbol, "H");
        assert_eq!(element_data(ElementId(6)).unwrap().symbol, "C");
        assert_eq!(element_data(ElementId(18)).unwrap().symbol, "Ar");
        assert!(element_data(ElementId(0)).is_none());
        assert!(element_data(ElementId(19)).is_none());
    }
}
