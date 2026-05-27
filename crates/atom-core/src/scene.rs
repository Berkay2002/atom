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
