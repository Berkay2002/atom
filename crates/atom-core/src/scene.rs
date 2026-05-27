//! Scene data model: a collection of `Atom`s plus a `View`.
//!
//! Slice 1 only ever instantiates a single hydrogen atom at the origin, but
//! the structure is shaped from day one for the multi-atom direction (see
//! `docs/superpowers/specs/2026-05-27-multi-atom-design.md`). Per that
//! spec, `Orbital` is a single (n, l, m) triple — *not* a `Vec` — because
//! a true in-atom superposition density is `|Σ c·ψ|²`, not `Σ |c·ψ|²`, and
//! we don't want the type to silently invite the wrong implementation.
//!
//! The `View` fields (`use_bare_z`, camera, colormap, exposure) exist as
//! placeholders for later slices (URL state, bare-Z toggle). No UI is
//! wired to them yet.

/// Identifier for an element. Today only hydrogen is supported; the full
/// element table arrives in issue 02. Kept as a `u32` atomic number so the
/// future expansion is a no-op for existing data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementId(pub u32);

impl ElementId {
    /// Hydrogen, Z = 1. The only element supported in slice 1.
    pub const HYDROGEN: ElementId = ElementId(1);
}

/// A single (n, l, m) orbital. Not a `Vec` — see module docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Orbital {
    pub n: u32,
    pub l: u32,
    pub m: i32,
}

/// One atom in a scene: an element, a position (in a₀), and the orbital
/// being visualized on it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Atom {
    pub element: ElementId,
    pub position: [f64; 3],
    pub orbital: Orbital,
}

/// Placeholder camera state. The desktop and web targets each keep their
/// own camera math (ADR-0001 carve-out); these fields exist so a future
/// shareable-URL slice can round-trip a viewing pose without yet another
/// type rename.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraState {
    pub azimuth: f32,
    pub elevation: f32,
    pub distance: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self { azimuth: 0.0, elevation: 0.0, distance: 1.0 }
    }
}

/// Placeholder colormap identifier. The actual LUTs live per-target (see
/// ADR-0001). This is a thin index so URL state and View are wire-compatible
/// across desktop and web in later slices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColormapId(pub u32);

/// View/presentation settings that apply to the scene as a whole. Note that
/// `use_bare_z` lives here (not on `Atom`): the bare-vs-effective `Z` choice
/// is a global "which lesson are we teaching" toggle, not a per-atom property.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub use_bare_z: bool,
    pub camera: CameraState,
    pub colormap: ColormapId,
    pub exposure: f32,
}

impl Default for View {
    fn default() -> Self {
        Self {
            use_bare_z: false,
            camera: CameraState::default(),
            colormap: ColormapId::default(),
            exposure: 1.0,
        }
    }
}

/// A scene is the unit the volume bake consumes: a set of atoms plus the
/// shared view state.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub atoms: Vec<Atom>,
    pub view: View,
}

impl Scene {
    /// Convenience constructor for slice 1's "one hydrogen atom at origin"
    /// case. Equivalent to the old `bake(n, l, m, res)` call shape.
    pub fn single_hydrogen(orbital: Orbital) -> Self {
        Self {
            atoms: vec![Atom {
                element: ElementId::HYDROGEN,
                position: [0.0, 0.0, 0.0],
                orbital,
            }],
            view: View::default(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
//  URL state codec — `v1:` format
// ─────────────────────────────────────────────────────────────────────────
//
// Single source of truth for the encoded scene string shared between
// `atom-desktop` and the WASM web target (issue 04). Lives here, in
// `atom-core`, so a URL produced by one client always decodes on the
// other.
//
// **Format (v1):** `v1:<Z>/<n>/<l>/<m>/<bare|eff>/<colormap_id>/<exposure>`
//
//   * `Z`             — atomic number, integer in `1..=18`
//   * `n`, `l`        — non-negative integers (`n >= 1`, `0 <= l < n`)
//   * `m`             — signed integer with `-l <= m <= l`
//   * `bare|eff`      — literal `"bare"` (use_bare_z=true) or `"eff"`
//   * `colormap_id`   — unsigned integer (matches `ColormapId.0`)
//   * `exposure`      — float with 2 decimals, e.g. `1.00`
//
// **Slice-1 scope.** Only single-atom scenes (one entry in `Scene::atoms`)
// at the origin round-trip. Multi-atom encoding is deliberately out of
// scope until issue 05 widens the JS↔WASM API. `encode` rejects multi-
// atom scenes with `EncodeError::UnsupportedMultiAtom` rather than
// silently dropping fields.
//
// **Camera state is NOT encoded.** Orbit-camera UX is "users drag to
// look around" — encoding camera state would write to the URL on every
// frame of a drag. `View::camera` exists in Rust but is intentionally
// absent from the wire format; decoded scenes carry `CameraState::default()`.

/// Errors produced by `decode`. `Display` is implemented so a UI banner can
/// render the message verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Input didn't start with a recognised version prefix.
    UnsupportedVersion(String),
    /// Too few `/`-delimited fields after the version prefix.
    MissingField { position: usize, field_name: &'static str },
    /// Atomic number outside the supported `1..=18` range.
    UnknownElement(u32),
    /// `(n, l, m)` triple violates the hydrogen-like quantum-number rules.
    InvalidQuantum { n: u32, l: u32, m: i32, reason: String },
    /// A field couldn't be parsed or was outside its allowed range.
    InvalidValue { field: &'static str, value: String },
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::UnsupportedVersion(v) => {
                write!(f, "unsupported version '{}'", v)
            }
            DecodeError::MissingField { position, field_name } => {
                write!(f, "missing field '{}' at position {}", field_name, position)
            }
            DecodeError::UnknownElement(z) => {
                write!(f, "unknown element Z={} (expected 1..=18)", z)
            }
            DecodeError::InvalidQuantum { n, l, m, reason } => {
                write!(f, "invalid quantum numbers (n={}, l={}, m={}): {}", n, l, m, reason)
            }
            DecodeError::InvalidValue { field, value } => {
                write!(f, "invalid value '{}' for field '{}'", value, field)
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Errors produced by `encode`. Multi-atom and non-origin scenes aren't
/// expressible in the slice-1 wire format; these surface as typed errors
/// rather than silent data loss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Scene has zero atoms — nothing to encode.
    EmptyScene,
    /// More than one atom; multi-atom encoding lands with issue 05.
    UnsupportedMultiAtom { count: usize },
}

impl core::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            EncodeError::EmptyScene => write!(f, "cannot encode an empty scene"),
            EncodeError::UnsupportedMultiAtom { count } => write!(
                f,
                "multi-atom scenes are not yet supported in the v1 URL format (got {} atoms)",
                count
            ),
        }
    }
}

impl std::error::Error for EncodeError {}

/// Field-name constants — kept in one place so the `MissingField` error
/// messages and the encoder stay in lock-step. The order matches the wire
/// format exactly.
const FIELD_NAMES: [&str; 7] = [
    "element_z",
    "n",
    "l",
    "m",
    "use_bare_z",
    "colormap_id",
    "exposure",
];

const VERSION_PREFIX: &str = "v1:";

/// Encode a `Scene` into the shareable `v1:` URL string.
///
/// See the module docs above for the exact format. Slice-1 only supports
/// single-atom scenes at the origin; richer scenes return
/// `EncodeError::UnsupportedMultiAtom` or `EncodeError::EmptyScene`.
pub fn encode(scene: &Scene) -> Result<String, EncodeError> {
    if scene.atoms.is_empty() {
        return Err(EncodeError::EmptyScene);
    }
    if scene.atoms.len() > 1 {
        return Err(EncodeError::UnsupportedMultiAtom { count: scene.atoms.len() });
    }
    let atom = &scene.atoms[0];
    let view = &scene.view;
    let bare = if view.use_bare_z { "bare" } else { "eff" };
    Ok(format!(
        "{}{}/{}/{}/{}/{}/{}/{:.2}",
        VERSION_PREFIX,
        atom.element.0,
        atom.orbital.n,
        atom.orbital.l,
        atom.orbital.m,
        bare,
        view.colormap.0,
        view.exposure,
    ))
}

/// Decode a `v1:` URL string back into a `Scene`. The decoded scene is
/// always a single atom at the origin (slice-1 scope); see the module
/// docs for the exact field order.
pub fn decode(s: &str) -> Result<Scene, DecodeError> {
    // Version gate. Anything other than the `v1:` prefix is rejected with
    // the offending token so the caller can show a useful message.
    let body = if let Some(rest) = s.strip_prefix(VERSION_PREFIX) {
        rest
    } else {
        // Extract the leading `vN:` token if present, otherwise echo the
        // full input — keeps "v2:..." readable and bare junk debuggable.
        let token = s.split(':').next().unwrap_or(s);
        return Err(DecodeError::UnsupportedVersion(token.to_string()));
    };

    let parts: Vec<&str> = body.split('/').collect();
    // Borrow at most the first 7 fields and bail clearly when one is
    // absent — the UI displays the field name, not just a numeric index.
    let get = |i: usize| -> Result<&str, DecodeError> {
        parts.get(i).copied().ok_or(DecodeError::MissingField {
            position: i,
            field_name: FIELD_NAMES[i],
        })
    };

    let z: u32 = get(0)?
        .parse()
        .map_err(|_| DecodeError::InvalidValue { field: FIELD_NAMES[0], value: parts.get(0).copied().unwrap_or("").to_string() })?;
    if !(1..=18).contains(&z) {
        return Err(DecodeError::UnknownElement(z));
    }

    let n: u32 = get(1)?
        .parse()
        .map_err(|_| DecodeError::InvalidValue { field: FIELD_NAMES[1], value: parts[1].to_string() })?;
    let l: u32 = get(2)?
        .parse()
        .map_err(|_| DecodeError::InvalidValue { field: FIELD_NAMES[2], value: parts[2].to_string() })?;
    let m: i32 = get(3)?
        .parse()
        .map_err(|_| DecodeError::InvalidValue { field: FIELD_NAMES[3], value: parts[3].to_string() })?;

    // Hydrogen-like quantum-number rules: n >= 1, 0 <= l < n, |m| <= l.
    if n < 1 {
        return Err(DecodeError::InvalidQuantum {
            n, l, m,
            reason: "n must be >= 1".to_string(),
        });
    }
    if l >= n {
        return Err(DecodeError::InvalidQuantum {
            n, l, m,
            reason: "l must satisfy 0 <= l < n".to_string(),
        });
    }
    if m.unsigned_abs() > l {
        return Err(DecodeError::InvalidQuantum {
            n, l, m,
            reason: "|m| must satisfy -l <= m <= l".to_string(),
        });
    }

    let bare_field = get(4)?;
    let use_bare_z = match bare_field {
        "bare" => true,
        "eff" => false,
        other => return Err(DecodeError::InvalidValue { field: FIELD_NAMES[4], value: other.to_string() }),
    };

    let colormap_id: u32 = get(5)?
        .parse()
        .map_err(|_| DecodeError::InvalidValue { field: FIELD_NAMES[5], value: parts[5].to_string() })?;

    let exposure: f32 = get(6)?
        .parse()
        .map_err(|_| DecodeError::InvalidValue { field: FIELD_NAMES[6], value: parts[6].to_string() })?;
    if !exposure.is_finite() {
        return Err(DecodeError::InvalidValue { field: FIELD_NAMES[6], value: parts[6].to_string() });
    }

    Ok(Scene {
        atoms: vec![Atom {
            element: ElementId(z),
            position: [0.0, 0.0, 0.0],
            orbital: Orbital { n, l, m },
        }],
        view: View {
            use_bare_z,
            camera: CameraState::default(),
            colormap: ColormapId(colormap_id),
            exposure,
        },
    })
}

#[cfg(test)]
mod codec_tests {
    use super::*;

    /// Build a representative scene by tweaking individual fields. Used by
    /// the round-trip table so each case stays focused on one axis.
    fn scene(z: u32, n: u32, l: u32, m: i32, bare: bool, cm: u32, exposure: f32) -> Scene {
        Scene {
            atoms: vec![Atom {
                element: ElementId(z),
                position: [0.0, 0.0, 0.0],
                orbital: Orbital { n, l, m },
            }],
            view: View {
                use_bare_z: bare,
                camera: CameraState::default(),
                colormap: ColormapId(cm),
                exposure,
            },
        }
    }

    #[test]
    fn round_trip_representative_scenes() {
        // Each row exercises a different axis of the codec — element, n/l/m,
        // bare-Z toggle, colormap, and exposure — so a regression on any
        // single field shows up as one named failure rather than a blob.
        let cases = vec![
            ("hydrogen 1s defaults",         scene(1, 1, 0,  0, false, 0, 1.00)),
            ("carbon 2p_z effective Z",      scene(6, 2, 1,  0, false, 3, 1.20)),
            ("oxygen 2p_z bare Z",           scene(8, 2, 1,  0, true,  0, 1.00)),
            ("argon 3d_(xy) bare + viridis", scene(18, 3, 2, -2, true, 1, 0.75)),
            ("nitrogen 4f_(z3) eff + magma", scene(7, 4, 3,  0, false, 2, 2.50)),
        ];
        for (label, original) in cases {
            let encoded = encode(&original).expect("encode succeeds");
            let decoded = decode(&encoded).unwrap_or_else(|e| {
                panic!("decode of '{}' failed for case '{}': {}", encoded, label, e)
            });
            assert_eq!(decoded, original, "round-trip mismatch for case '{}': {}", label, encoded);
        }
    }

    #[test]
    fn fixed_string_fidelity_locks_format() {
        // This test pins the wire format byte-for-byte. If a future edit
        // shifts field order, renames `bare`/`eff`, or changes the
        // exposure precision, this will fail — which is the intent: the
        // desktop and the web *must* agree on every byte.
        let url = "v1:1/1/0/0/eff/0/1.00";
        let decoded = decode(url).expect("hand-written URL decodes");

        assert_eq!(decoded.atoms.len(), 1);
        let atom = &decoded.atoms[0];
        assert_eq!(atom.element, ElementId(1));
        assert_eq!(atom.position, [0.0, 0.0, 0.0]);
        assert_eq!(atom.orbital, Orbital { n: 1, l: 0, m: 0 });

        assert!(!decoded.view.use_bare_z);
        assert_eq!(decoded.view.colormap, ColormapId(0));
        assert_eq!(decoded.view.exposure, 1.00);

        // And the inverse — encoding the decoded scene reproduces the
        // exact original string, no whitespace, no trailing slash.
        assert_eq!(encode(&decoded).unwrap(), url);
    }

    #[test]
    fn rejects_unsupported_version() {
        let err = decode("v2:1/1/0/0/eff/0/1.00").unwrap_err();
        assert_eq!(err, DecodeError::UnsupportedVersion("v2".to_string()));
        let msg = format!("{}", err);
        assert!(msg.contains("v2"), "display message should mention version: {}", msg);
    }

    #[test]
    fn rejects_unknown_element() {
        // Z=42 isn't in the H–Ar table.
        let err = decode("v1:42/1/0/0/eff/0/1.00").unwrap_err();
        assert_eq!(err, DecodeError::UnknownElement(42));
    }

    #[test]
    fn rejects_missing_fields() {
        // Drop the exposure field — the decoder must point at the named
        // field that's missing, not a numeric index.
        let err = decode("v1:1/1/0/0/eff/0").unwrap_err();
        assert_eq!(
            err,
            DecodeError::MissingField { position: 6, field_name: "exposure" }
        );
    }

    #[test]
    fn rejects_invalid_quantum_numbers() {
        // n=1 only permits l=0; l=2 violates 0 <= l < n.
        let err = decode("v1:1/1/2/0/eff/0/1.00").unwrap_err();
        match err {
            DecodeError::InvalidQuantum { n, l, .. } => {
                assert_eq!(n, 1);
                assert_eq!(l, 2);
            }
            other => panic!("expected InvalidQuantum, got {:?}", other),
        }
    }

    #[test]
    fn rejects_invalid_bare_token() {
        let err = decode("v1:1/1/0/0/maybe/0/1.00").unwrap_err();
        assert_eq!(
            err,
            DecodeError::InvalidValue { field: "use_bare_z", value: "maybe".to_string() }
        );
    }

    #[test]
    fn encode_rejects_empty_scene() {
        let s = Scene { atoms: vec![], view: View::default() };
        assert_eq!(encode(&s), Err(EncodeError::EmptyScene));
    }

    #[test]
    fn encode_rejects_multi_atom_scene() {
        let s = Scene {
            atoms: vec![
                Atom { element: ElementId(1), position: [0.0; 3], orbital: Orbital { n: 1, l: 0, m: 0 } },
                Atom { element: ElementId(1), position: [1.0, 0.0, 0.0], orbital: Orbital { n: 1, l: 0, m: 0 } },
            ],
            view: View::default(),
        };
        assert_eq!(encode(&s), Err(EncodeError::UnsupportedMultiAtom { count: 2 }));
    }
}
