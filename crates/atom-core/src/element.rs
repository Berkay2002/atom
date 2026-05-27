//! Static table of element data for the first 18 elements (H–Ar).
//!
//! Each entry carries the symbol, name, atomic number, ground-state
//! electron configuration (machine-readable), and a human-readable config
//! string for captions. Lookup is by `ElementId` (atomic number).
//!
//! This module also owns the UI-ready presentation facts shared by the
//! desktop and web targets: periodic-table slot metadata, the canonical
//! display name, the electron-configuration display string, and the HOMO
//! orbital snap target for each supported element.
//!
//! The electron configuration is stored as a slice of `(n, l, count)`
//! triples — `slater.rs` consumes this directly when resolving the
//! per-orbital screening sum.

use crate::scene::{ElementId, Orbital, Scene};

/// A single `(n, l, count)` entry in an electron configuration.
///
/// `n` is the principal quantum number, `l` is the orbital angular
/// momentum (0=s, 1=p, 2=d, 3=f), `count` is the number of electrons
/// occupying that subshell in the element's ground state.
pub type ConfigEntry = (u32, u32, u32);

/// Periodic-table placement for a supported element.
///
/// `period` is the row number and `group` is the 1-based column in the
/// standard 18-column periodic-table layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodicSlot {
    pub period: u8,
    pub group: u8,
}

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

/// UI-ready presentation facts for one supported element.
///
/// This is the shared lookup surface for the element picker, captions,
/// and HOMO snap behavior. It deliberately stays data-only: target UIs
/// can paint the facts however they want, while the core projects facts
/// from the canonical `ELEMENTS` table plus the periodic-table slot map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementPresentation {
    pub atomic_number: u32,
    pub symbol: &'static str,
    pub display_name: &'static str,
    pub config_text: &'static str,
    pub homo: Orbital,
    pub slot: PeriodicSlot,
}

/// Periodic-table slots for the supported H–Ar elements, in atomic-number
/// order.
///
/// This is the only extra element-presentation data the core stores.
/// Everything else is projected from `ELEMENTS` and `homo()`.
pub const PERIODIC_SLOTS: [PeriodicSlot; 18] = [
    PeriodicSlot {
        period: 1,
        group: 1,
    },
    PeriodicSlot {
        period: 1,
        group: 18,
    },
    PeriodicSlot {
        period: 2,
        group: 1,
    },
    PeriodicSlot {
        period: 2,
        group: 2,
    },
    PeriodicSlot {
        period: 2,
        group: 13,
    },
    PeriodicSlot {
        period: 2,
        group: 14,
    },
    PeriodicSlot {
        period: 2,
        group: 15,
    },
    PeriodicSlot {
        period: 2,
        group: 16,
    },
    PeriodicSlot {
        period: 2,
        group: 17,
    },
    PeriodicSlot {
        period: 2,
        group: 18,
    },
    PeriodicSlot {
        period: 3,
        group: 1,
    },
    PeriodicSlot {
        period: 3,
        group: 2,
    },
    PeriodicSlot {
        period: 3,
        group: 13,
    },
    PeriodicSlot {
        period: 3,
        group: 14,
    },
    PeriodicSlot {
        period: 3,
        group: 15,
    },
    PeriodicSlot {
        period: 3,
        group: 16,
    },
    PeriodicSlot {
        period: 3,
        group: 17,
    },
    PeriodicSlot {
        period: 3,
        group: 18,
    },
];

// Ground-state configurations for H–Ar. Build-up order is Madelung-correct
// for these 18 elements (the 4s/3d crossover doesn't appear until K/Ca,
// which we exclude).
const C_H: &[ConfigEntry] = &[(1, 0, 1)];
const C_HE: &[ConfigEntry] = &[(1, 0, 2)];
const C_LI: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 1)];
const C_BE: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2)];
const C_B: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 1)];
const C_C: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 2)];
const C_N: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 3)];
const C_O: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 4)];
const C_F: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 5)];
const C_NE: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6)];
const C_NA: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 1)];
const C_MG: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2)];
const C_AL: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 1)];
const C_SI: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 2)];
const C_P: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 3)];
const C_S: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 4)];
const C_CL: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 5)];
const C_AR: &[ConfigEntry] = &[(1, 0, 2), (2, 0, 2), (2, 1, 6), (3, 0, 2), (3, 1, 6)];

/// Table of all 18 supported elements (H through Ar), in atomic-number
/// order. Index `i` holds element with atomic number `i + 1`.
pub const ELEMENTS: [ElementData; 18] = [
    ElementData {
        symbol: "H",
        name: "Hydrogen",
        atomic_number: 1,
        electron_config: C_H,
        config_str: "1s\u{00B9}",
    },
    ElementData {
        symbol: "He",
        name: "Helium",
        atomic_number: 2,
        electron_config: C_HE,
        config_str: "1s\u{00B2}",
    },
    ElementData {
        symbol: "Li",
        name: "Lithium",
        atomic_number: 3,
        electron_config: C_LI,
        config_str: "1s\u{00B2} 2s\u{00B9}",
    },
    ElementData {
        symbol: "Be",
        name: "Beryllium",
        atomic_number: 4,
        electron_config: C_BE,
        config_str: "1s\u{00B2} 2s\u{00B2}",
    },
    ElementData {
        symbol: "B",
        name: "Boron",
        atomic_number: 5,
        electron_config: C_B,
        config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{00B9}",
    },
    ElementData {
        symbol: "C",
        name: "Carbon",
        atomic_number: 6,
        electron_config: C_C,
        config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{00B2}",
    },
    ElementData {
        symbol: "N",
        name: "Nitrogen",
        atomic_number: 7,
        electron_config: C_N,
        config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{00B3}",
    },
    ElementData {
        symbol: "O",
        name: "Oxygen",
        atomic_number: 8,
        electron_config: C_O,
        config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{2074}",
    },
    ElementData {
        symbol: "F",
        name: "Fluorine",
        atomic_number: 9,
        electron_config: C_F,
        config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{2075}",
    },
    ElementData {
        symbol: "Ne",
        name: "Neon",
        atomic_number: 10,
        electron_config: C_NE,
        config_str: "1s\u{00B2} 2s\u{00B2} 2p\u{2076}",
    },
    ElementData {
        symbol: "Na",
        name: "Sodium",
        atomic_number: 11,
        electron_config: C_NA,
        config_str: "[Ne] 3s\u{00B9}",
    },
    ElementData {
        symbol: "Mg",
        name: "Magnesium",
        atomic_number: 12,
        electron_config: C_MG,
        config_str: "[Ne] 3s\u{00B2}",
    },
    ElementData {
        symbol: "Al",
        name: "Aluminium",
        atomic_number: 13,
        electron_config: C_AL,
        config_str: "[Ne] 3s\u{00B2} 3p\u{00B9}",
    },
    ElementData {
        symbol: "Si",
        name: "Silicon",
        atomic_number: 14,
        electron_config: C_SI,
        config_str: "[Ne] 3s\u{00B2} 3p\u{00B2}",
    },
    ElementData {
        symbol: "P",
        name: "Phosphorus",
        atomic_number: 15,
        electron_config: C_P,
        config_str: "[Ne] 3s\u{00B2} 3p\u{00B3}",
    },
    ElementData {
        symbol: "S",
        name: "Sulfur",
        atomic_number: 16,
        electron_config: C_S,
        config_str: "[Ne] 3s\u{00B2} 3p\u{2074}",
    },
    ElementData {
        symbol: "Cl",
        name: "Chlorine",
        atomic_number: 17,
        electron_config: C_CL,
        config_str: "[Ne] 3s\u{00B2} 3p\u{2075}",
    },
    ElementData {
        symbol: "Ar",
        name: "Argon",
        atomic_number: 18,
        electron_config: C_AR,
        config_str: "[Ne] 3s\u{00B2} 3p\u{2076}",
    },
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

/// Look up the shared presentation facts for an element by atomic number.
///
/// Returns `None` for any `ElementId` outside `1..=18`. Callers should
/// keep using the existing caption fallback (`Element Z=<n>`) when they
/// need a readable string for unsupported elements.
pub fn element_presentation(id: ElementId) -> Option<ElementPresentation> {
    let data = element_data(id)?;
    let homo = homo(id)?;
    let slot = *PERIODIC_SLOTS.get((id.0 - 1) as usize)?;
    Some(ElementPresentation {
        atomic_number: data.atomic_number,
        symbol: data.symbol,
        display_name: data.name,
        config_text: data.config_str,
        homo,
        slot,
    })
}

/// Highest occupied orbital for an element, picked as `(n, l, m = 0)`.
///
/// The `(n, l)` pair is the last entry of `electron_config`. For H–Ar
/// the build-up order is Madelung-correct with no 4s/3d crossover, so
/// "last entry" coincides with the highest-energy occupied subshell.
///
/// `m = 0` is the textbook "vertical" choice — `s` for `l=0`, `p_z` for
/// `l=1`, `d_z²` for `l=2` — always within `|m| ≤ l` and the most
/// pedagogically iconic orientation when ray-marched.
///
/// Returns `None` for unknown elements (Z outside 1..=18).
pub fn homo(element: ElementId) -> Option<Orbital> {
    let data = element_data(element)?;
    // Every entry in `ELEMENTS` has a non-empty config (asserted by the
    // `electron_counts_sum_to_atomic_number` test), so `.last()` is
    // infallible in practice — but expressing it as `?` keeps the helper
    // total for any future ElementData with an empty config.
    let &(n, l, _) = data.electron_config.last()?;
    Some(Orbital { n, l, m: 0 })
}

// ─────────────────────────────────────────────────────────────────────────
//  Plain-language caption for the "what am I looking at" UI strip
// ─────────────────────────────────────────────────────────────────────────
//
// `caption(&Scene) -> String` produces a single line that names what's on
// screen in spectroscopic notation plus a one-sentence gloss. Lives next
// to `ElementData` so the orbital-label table and per-orbital descriptions
// don't drift across the web/desktop boundary — both targets call into
// the same function via either a direct `pub use` (desktop) or the
// `scene_caption` wasm-bindgen export (web).
//
// Slice-1 only handles the single-atom case (the only shape a `Scene`
// can take today). Multi-atom captions are a follow-up — the function
// falls back to "<n atoms>" when handed more than one, which keeps the
// caller from rendering an empty string.

/// Spectroscopic letter for an angular-momentum quantum number `l`.
/// Returns `s`, `p`, `d`, `f`, `g`, `h`, … (a-z after `h`); higher `l`
/// values use lowercase letters via the standard convention.
fn l_letter(l: u32) -> &'static str {
    // Standard spectroscopic series. Beyond `h` the convention is to
    // continue alphabetically (skipping `j` so the letter `i` isn't
    // confused with the imaginary unit), but no element in our table
    // populates an `l >= 6` orbital so the fallback never fires.
    match l {
        0 => "s",
        1 => "p",
        2 => "d",
        3 => "f",
        4 => "g",
        5 => "h",
        _ => "?",
    }
}

/// Real-spherical-harmonic label for an `(l, m)` pair. Returns the bare
/// spectroscopic letter for `l=0`, the standard p/d cartesian labels for
/// `l=1`/`l=2`, and `None` for `(l, m)` combinations outside the slice-1
/// coverage (which the caller renders as just `n` + `l_letter`).
///
/// Naming convention follows the common chemistry-textbook real spherical
/// harmonics:
///
/// | l | m   | label    |
/// |---|-----|----------|
/// | 0 |  0  | s        |
/// | 1 | -1  | p_y      |
/// | 1 |  0  | p_z      |
/// | 1 | +1  | p_x      |
/// | 2 | -2  | d_xy     |
/// | 2 | -1  | d_yz     |
/// | 2 |  0  | d_z²     |
/// | 2 | +1  | d_xz     |
/// | 2 | +2  | d_x²-y²  |
pub fn orbital_label(l: u32, m: i32) -> Option<&'static str> {
    match (l, m) {
        (0, 0) => Some("s"),
        (1, -1) => Some("p_y"),
        (1, 0) => Some("p_z"),
        (1, 1) => Some("p_x"),
        (2, -2) => Some("d_xy"),
        (2, -1) => Some("d_yz"),
        (2, 0) => Some("d_z\u{00B2}"),
        (2, 1) => Some("d_xz"),
        (2, 2) => Some("d_x\u{00B2}-y\u{00B2}"),
        _ => None,
    }
}

/// One-sentence plain-language description for an `(n, l)` subshell. Six
/// hand-written entries cover the slice-1 menu (1s, 2s, 2p, 3s, 3p, 3d).
/// Higher (n, l) combinations fall back to a generic phrasing in
/// `caption` so unknown orbitals still produce a readable line.
pub fn orbital_description(n: u32, l: u32) -> Option<&'static str> {
    match (n, l) {
        (1, 0) => Some("the ground-state orbital, spherical and centered on the nucleus."),
        (2, 0) => Some("a larger spherical orbital with one radial node."),
        (2, 1) => Some("a dumbbell-shaped orbital with two lobes along one axis."),
        (3, 0) => Some("a spherical orbital with two radial nodes — the 3s shell."),
        (3, 1) => Some("a dumbbell-shaped 3p orbital, larger than 2p with an extra radial node."),
        (3, 2) => {
            Some("a four-lobed d orbital — the first shell where the cloverleaf shapes appear.")
        }
        _ => None,
    }
}

/// Compose the user-facing caption for a `Scene`.
///
/// Format for a single-atom scene:
///   `"<Element name> <n><orbital-label> — <one-sentence description>"`
///
/// Edge cases:
///   * Empty scene → `"No atoms"`.
///   * Multi-atom scene → `"<count> atoms"` (placeholder until the
///     multi-atom UI lands; keeps the caller from rendering blank).
///   * Unknown element Z → falls back to `"Element Z=<n>"`.
///   * `(l, m)` outside the labeled table → drops the subscript and uses
///     just `"<n><letter>"` (e.g. `"4f"`), so exotic orbitals still read
///     cleanly even when no cartesian label is defined.
///   * `(n, l)` outside the description table → generic
///     `"<element-name> <orbital-label> orbital."` line.
pub fn caption(scene: &Scene) -> String {
    match scene.atoms.len() {
        0 => "No atoms".to_string(),
        1 => single_atom_caption(scene),
        n => format!("{} atoms", n),
    }
}

fn single_atom_caption(scene: &Scene) -> String {
    let atom = &scene.atoms[0];
    let (n, l, m) = (atom.orbital.n, atom.orbital.l, atom.orbital.m);

    // Pull the element's friendly name; fall back to `Z=<n>` so a
    // future element-id outside the table still produces a line that
    // makes sense rather than crashing the UI.
    let name = element_data(atom.element)
        .map(|e| e.name.to_string())
        .unwrap_or_else(|| format!("Element Z={}", atom.element.0));

    // Spectroscopic name. Use the cartesian label if we have one,
    // otherwise just `<n><letter>`.
    let orbital_name = match orbital_label(l, m) {
        Some(label) => format!("{}{}", n, label),
        None => format!("{}{}", n, l_letter(l)),
    };

    let description = orbital_description(n, l)
        .map(|d| d.to_string())
        // Generic fallback for orbitals outside the hand-written table.
        // Avoid the "a/an" article problem by phrasing as a noun phrase:
        // "<element> <n><letter> orbital." reads naturally for any (n, l).
        .unwrap_or_else(|| format!("{} orbital.", orbital_name));

    format!("{} {} \u{2014} {}", name, orbital_name, description)
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
            assert_eq!(
                total, e.atomic_number,
                "{} electron count mismatch",
                e.symbol
            );
        }
    }

    #[test]
    fn homo_matches_textbook_homo_for_h_through_ar() {
        // Iconic-orbital snap for the periodic-table picker: when the
        // user clicks an element, the UI jumps to its HOMO so the cloud
        // is never empty. Locking the expected table here so any future
        // edit to ELEMENTS that shifts the build-up order fails loudly.
        let cases: &[(u32, u32, u32)] = &[
            (1, 1, 0),  // H  → 1s
            (2, 1, 0),  // He → 1s
            (3, 2, 0),  // Li → 2s
            (4, 2, 0),  // Be → 2s
            (5, 2, 1),  // B  → 2p
            (6, 2, 1),  // C  → 2p
            (7, 2, 1),  // N  → 2p
            (8, 2, 1),  // O  → 2p
            (9, 2, 1),  // F  → 2p
            (10, 2, 1), // Ne → 2p
            (11, 3, 0), // Na → 3s
            (12, 3, 0), // Mg → 3s
            (13, 3, 1), // Al → 3p
            (14, 3, 1), // Si → 3p
            (15, 3, 1), // P  → 3p
            (16, 3, 1), // S  → 3p
            (17, 3, 1), // Cl → 3p
            (18, 3, 1), // Ar → 3p
        ];
        for &(z, n, l) in cases {
            let o = homo(ElementId(z)).expect("z in 1..=18");
            assert_eq!((o.n, o.l, o.m), (n, l, 0), "Z={z} HOMO mismatch");
        }
    }

    #[test]
    fn homo_returns_none_for_unknown_z() {
        assert!(homo(ElementId(0)).is_none());
        assert!(homo(ElementId(19)).is_none());
        assert!(homo(ElementId(99)).is_none());
    }

    #[test]
    fn lookup_by_id() {
        assert_eq!(element_data(ElementId(1)).unwrap().symbol, "H");
        assert_eq!(element_data(ElementId(6)).unwrap().symbol, "C");
        assert_eq!(element_data(ElementId(18)).unwrap().symbol, "Ar");
        assert!(element_data(ElementId(0)).is_none());
        assert!(element_data(ElementId(19)).is_none());
    }

    #[test]
    fn presentation_lookup_matches_domain_projection() {
        assert_eq!(ELEMENTS.len(), 18);
        assert_eq!(PERIODIC_SLOTS.len(), 18);
        for (i, data) in ELEMENTS.iter().enumerate() {
            let z = (i + 1) as u32;
            let actual = element_presentation(ElementId(z)).expect("supported element");
            assert_eq!(
                actual.atomic_number, data.atomic_number,
                "Z={z} atomic number"
            );
            assert_eq!(actual.symbol, data.symbol, "Z={z} symbol");
            assert_eq!(actual.display_name, data.name, "Z={z} display name");
            assert_eq!(actual.config_text, data.config_str, "Z={z} config text");
            assert_eq!(actual.homo, homo(ElementId(z)).unwrap(), "Z={z} HOMO");
            assert_eq!(actual.slot, PERIODIC_SLOTS[i], "Z={z} slot");
        }
    }

    #[test]
    fn presentation_lookup_returns_none_for_unknown_z() {
        assert!(element_presentation(ElementId(0)).is_none());
        assert!(element_presentation(ElementId(19)).is_none());
        assert!(element_presentation(ElementId(99)).is_none());
    }

    #[test]
    fn presentation_symbols_and_slots_are_unique() {
        let mut symbols: HashSet<&str> = HashSet::new();
        let mut slots: HashSet<(u8, u8)> = HashSet::new();

        for e in ELEMENTS.iter() {
            assert!(symbols.insert(e.symbol), "duplicate symbol {}", e.symbol);
        }
        for slot in PERIODIC_SLOTS.iter() {
            assert!(
                slots.insert((slot.period, slot.group)),
                "duplicate slot {}:{}",
                slot.period,
                slot.group
            );
            assert!(
                (1..=3).contains(&slot.period),
                "invalid period {}",
                slot.period
            );
            assert!(
                (1..=18).contains(&slot.group),
                "invalid group {}",
                slot.group
            );
        }

        assert_eq!(symbols.len(), 18);
        assert_eq!(slots.len(), 18);
    }

    #[test]
    fn presentation_fallback_is_explicit_for_unknown_elements() {
        assert!(element_presentation(ElementId(0)).is_none());
        assert!(element_presentation(ElementId(19)).is_none());
        assert!(element_presentation(ElementId(99)).is_none());

        let s = scene_single(99, 1, 0, 0);
        assert!(caption(&s).contains("Element Z=99"));
    }

    // ─── caption / orbital_label / orbital_description ────────────────

    use crate::scene::{Atom, Orbital, Scene, View};

    fn scene_single(z: u32, n: u32, l: u32, m: i32) -> Scene {
        Scene {
            atoms: vec![Atom {
                element: ElementId(z),
                position: [0.0, 0.0, 0.0],
                orbital: Orbital { n, l, m },
            }],
            view: View::default(),
        }
    }

    #[test]
    fn orbital_label_matches_standard_real_harmonic_table() {
        // s shell.
        assert_eq!(orbital_label(0, 0), Some("s"));
        // p shell: real spherical harmonics conventionally map
        // m=-1 → y, m=0 → z, m=+1 → x.
        assert_eq!(orbital_label(1, -1), Some("p_y"));
        assert_eq!(orbital_label(1, 0), Some("p_z"));
        assert_eq!(orbital_label(1, 1), Some("p_x"));
        // d shell — full cloverleaf set.
        assert_eq!(orbital_label(2, -2), Some("d_xy"));
        assert_eq!(orbital_label(2, -1), Some("d_yz"));
        assert_eq!(orbital_label(2, 0), Some("d_z\u{00B2}"));
        assert_eq!(orbital_label(2, 1), Some("d_xz"));
        assert_eq!(orbital_label(2, 2), Some("d_x\u{00B2}-y\u{00B2}"));
    }

    #[test]
    fn orbital_label_returns_none_for_uncovered_combos() {
        // No `f_*` cartesian labels in the table — slice 1's coverage
        // stops at d. The caller falls back to "<n><letter>".
        assert_eq!(orbital_label(3, 0), None);
        assert_eq!(orbital_label(3, 1), None);
        assert_eq!(orbital_label(3, -3), None);
        // And bogus (l, m) pairs that violate |m| <= l also fail
        // gracefully — orbital_label is pure-lookup and doesn't validate.
        assert_eq!(orbital_label(1, 5), None);
    }

    #[test]
    fn orbital_description_covers_at_least_six_subshells() {
        // Acceptance criterion: 6+ hand-written descriptions.
        let covered: &[(u32, u32)] = &[(1, 0), (2, 0), (2, 1), (3, 0), (3, 1), (3, 2)];
        for (n, l) in covered {
            assert!(
                orbital_description(*n, *l).is_some(),
                "expected description for (n={}, l={})",
                n,
                l
            );
        }
    }

    #[test]
    fn orbital_description_is_none_for_unknown_combinations() {
        // Higher-n combinations have no hand-written description and
        // must fall through to the generic fallback in `caption`.
        assert!(orbital_description(4, 3).is_none());
        assert!(orbital_description(5, 2).is_none());
    }

    #[test]
    fn caption_hydrogen_1s_uses_named_description() {
        // Hydrogen 1s — the canonical "what is this" intro line. Must
        // include the element name, the spectroscopic label, and the
        // ground-state phrase verbatim.
        let s = scene_single(1, 1, 0, 0);
        let c = caption(&s);
        assert!(c.starts_with("Hydrogen 1s"), "got: {}", c);
        assert!(c.contains("ground-state"), "got: {}", c);
        // Em-dash separator between the name and the description.
        assert!(c.contains('\u{2014}'), "expected em-dash in: {}", c);
    }

    #[test]
    fn caption_neon_2p_z_uses_cartesian_label() {
        let s = scene_single(10, 2, 1, 0);
        let c = caption(&s);
        assert!(c.starts_with("Neon 2p_z"), "got: {}", c);
        // 2p subshell description hits the dumbbell wording.
        assert!(c.contains("dumbbell"), "got: {}", c);
    }

    #[test]
    fn caption_carbon_2p_x_uses_cartesian_label() {
        let s = scene_single(6, 2, 1, 1);
        let c = caption(&s);
        assert!(c.starts_with("Carbon 2p_x"), "got: {}", c);
    }

    #[test]
    fn caption_unknown_orbital_combination_falls_back_gracefully() {
        // Carbon 4f — no orbital_label entry, no orbital_description entry.
        // Caption must not panic and must produce a readable line that
        // includes both the element and the bare "<n><letter>" form.
        let s = scene_single(6, 4, 3, 0);
        let c = caption(&s);
        assert!(c.starts_with("Carbon 4f"), "got: {}", c);
        // Generic fallback ends with "orbital." — the description table
        // didn't hit, so the format!() path kicks in.
        assert!(c.contains("orbital."), "got: {}", c);
    }

    #[test]
    fn caption_unknown_element_does_not_panic() {
        // Z=99 isn't in our table; caption should fall back to "Element Z=99"
        // rather than crashing or producing an empty string.
        let s = scene_single(99, 1, 0, 0);
        let c = caption(&s);
        assert!(c.contains("Z=99"), "got: {}", c);
    }

    #[test]
    fn caption_empty_scene_is_non_broken() {
        let s = Scene {
            atoms: vec![],
            view: View::default(),
        };
        assert_eq!(caption(&s), "No atoms");
    }

    /// Diagnostic helper — `cargo test print_caption_samples -- --ignored
    /// --nocapture` prints representative captions for manual review.
    /// Not part of the normal CI run; kept to make it easy to eyeball
    /// the format after future edits to the description table.
    #[test]
    #[ignore]
    fn print_caption_samples() {
        let cases = [
            (1, 1, 0, 0),
            (10, 2, 1, 0),
            (6, 2, 1, 1),
            (8, 2, 1, -1),
            (18, 3, 2, 0),
            (6, 4, 3, 0),
        ];
        for (z, n, l, m) in cases {
            let s = scene_single(z, n, l, m);
            eprintln!("Z={} n={} l={} m={:+}: {}", z, n, l, m, caption(&s));
        }
    }

    #[test]
    fn caption_multi_atom_scene_falls_back_to_count() {
        let s = Scene {
            atoms: vec![
                Atom {
                    element: ElementId(1),
                    position: [0.0; 3],
                    orbital: Orbital { n: 1, l: 0, m: 0 },
                },
                Atom {
                    element: ElementId(1),
                    position: [1.0, 0.0, 0.0],
                    orbital: Orbital { n: 1, l: 0, m: 0 },
                },
            ],
            view: View::default(),
        };
        assert_eq!(caption(&s), "2 atoms");
    }
}
