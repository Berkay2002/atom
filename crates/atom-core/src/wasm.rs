//! wasm-bindgen entry point for the volume bake.
//!
//! The browser-facing API is intentionally tiny. The single-atom shape
//! still flows as a thin parameter tuple — `(element_z, n, l, m,
//! use_bare_z, res)` — because the JS UI only configures one atom today.
//! Multi-atom scenes will widen this once issue 05 (richer scene UI)
//! lands.

use js_sys::Float32Array;
use wasm_bindgen::prelude::*;

use crate::element;
use crate::scene::{self, Atom, ElementId, Orbital, Scene, View};
use crate::volume;

/// Result of a single volume bake, owned on the Rust side.
///
/// `data` is exposed to JS as a zero-copy view into wasm memory; the view
/// is invalidated once this object is dropped (or wasm memory grows), so
/// callers should consume it immediately.
#[wasm_bindgen]
pub struct BakeResult {
    data: Vec<f32>,
    half_extent: f32,
    peak: f32,
}

#[wasm_bindgen]
impl BakeResult {
    /// Zero-copy view of the voxel grid (length = res³, row-major x→y→z).
    ///
    /// Safety: the returned `Float32Array` aliases wasm linear memory.
    /// JS must copy or upload its contents before this `BakeResult` is
    /// dropped or any subsequent allocation triggers memory growth.
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Float32Array {
        // SAFETY: We hand JS a view into our owned Vec<f32>. The view
        // becomes dangling if `self` is dropped or wasm memory grows
        // (which can move the heap). Per the contract above, callers
        // consume the view before either happens.
        unsafe { Float32Array::view(&self.data) }
    }

    /// Half-edge of the cubic bounding box, in Bohr radii.
    #[wasm_bindgen(getter)]
    pub fn half_extent(&self) -> f32 {
        self.half_extent
    }

    /// Absolute peak |ψ|² before normalization (for HUD display).
    #[wasm_bindgen(getter)]
    pub fn peak(&self) -> f32 {
        self.peak
    }
}

/// Bake a Scene containing a single atom of `element_z` at the origin for
/// orbital `(n, l, m)` at the given grid resolution.
///
/// `element_z` is the atomic number (1..=18 for H–Ar; out-of-range values
/// fall back to bare hydrogen inside Slater's-rules resolution).
///
/// When `use_bare_z` is `true`, the bake uses the element's bare atomic
/// number directly instead of the Slater-shielded effective charge. The
/// UI toggle for this lands in issue 03; the plumbing exists now so the
/// `View` is faithfully threaded across the JS↔WASM boundary.
#[wasm_bindgen]
pub fn bake_scene(
    element_z: u32,
    n: u32,
    l: u32,
    m: i32,
    use_bare_z: bool,
    res: u32,
) -> BakeResult {
    let scene = Scene {
        atoms: vec![Atom {
            element: ElementId(element_z),
            position: [0.0, 0.0, 0.0],
            orbital: Orbital { n, l, m },
        }],
        view: View { use_bare_z, ..View::default() },
    };
    let v = volume::bake_scene(&scene, res as usize);
    BakeResult {
        data: v.data,
        half_extent: v.half_extent as f32,
        peak: v.peak as f32,
    }
}

// ─────────────────────────────────────────────────────────────────────────
//  URL-state codec (issue 04)
// ─────────────────────────────────────────────────────────────────────────
//
// JS-friendly projection of a `Scene` for the slice-1 wire format. We
// deliberately don't expose the full `Scene` across the FFI boundary —
// it'd require leaking `Vec<Atom>` machinery into wasm-bindgen. Instead,
// the JS side hands us flat primitives, and we hand back a plain struct
// with getters for each field the URL actually carries. This keeps the
// FFI surface aligned with the wire format and makes drift impossible.

/// Decoded URL state, mirroring the wire-format field set 1:1. Position
/// is implicit (slice-1 scenes are single-atom at the origin), camera
/// state is excluded by design — see scene.rs module docs.
#[wasm_bindgen]
pub struct DecodedScene {
    element_z: u32,
    n: u32,
    l: u32,
    m: i32,
    use_bare_z: bool,
    colormap_id: u32,
    exposure: f32,
}

#[wasm_bindgen]
impl DecodedScene {
    #[wasm_bindgen(getter)]
    pub fn element_z(&self) -> u32 { self.element_z }
    #[wasm_bindgen(getter)]
    pub fn n(&self) -> u32 { self.n }
    #[wasm_bindgen(getter)]
    pub fn l(&self) -> u32 { self.l }
    #[wasm_bindgen(getter)]
    pub fn m(&self) -> i32 { self.m }
    #[wasm_bindgen(getter)]
    pub fn use_bare_z(&self) -> bool { self.use_bare_z }
    #[wasm_bindgen(getter)]
    pub fn colormap_id(&self) -> u32 { self.colormap_id }
    #[wasm_bindgen(getter)]
    pub fn exposure(&self) -> f32 { self.exposure }
}

/// Encode the slice-1 single-atom view to a `v1:` URL string. Always
/// succeeds — the input shape (flat primitives) can't represent a
/// multi-atom or empty scene, so the `EncodeError` variants are
/// unreachable here.
#[wasm_bindgen]
pub fn scene_encode(
    element_z: u32,
    n: u32,
    l: u32,
    m: i32,
    use_bare_z: bool,
    colormap_id: u32,
    exposure: f32,
) -> String {
    let scene = Scene {
        atoms: vec![Atom {
            element: ElementId(element_z),
            position: [0.0, 0.0, 0.0],
            orbital: Orbital { n, l, m },
        }],
        view: View {
            use_bare_z,
            camera: crate::scene::CameraState::default(),
            colormap: crate::scene::ColormapId(colormap_id),
            exposure,
        },
    };
    // unwrap: single non-empty atom — neither EncodeError variant can fire.
    scene::encode(&scene).expect("single-atom scene always encodes")
}

/// Compose the user-facing caption (issue 06) for the slice-1 single-atom
/// view. Mirrors `scene_encode`'s flat-primitive parameter shape so the
/// JS side never has to construct a `Scene` across the FFI boundary.
///
/// The returned string is the same line `atom_core::caption(&scene)`
/// produces for the equivalent single-atom `Scene`.
#[wasm_bindgen]
pub fn scene_caption(element_z: u32, n: u32, l: u32, m: i32) -> String {
    let scene = Scene {
        atoms: vec![Atom {
            element: ElementId(element_z),
            position: [0.0, 0.0, 0.0],
            orbital: Orbital { n, l, m },
        }],
        view: View::default(),
    };
    crate::element::caption(&scene)
}

/// JS-friendly projection of `atom_core::element_presentation`.
///
/// The web target keeps a synchronous TS snapshot for click handling and
/// layout, but the parity tests compare that snapshot against this
/// wasm-exported Rust projection so the snapshot cannot drift.
#[wasm_bindgen]
pub struct ElementPresentationJs {
    atomic_number: u32,
    symbol: &'static str,
    display_name: &'static str,
    config_text: &'static str,
    homo_n: u32,
    homo_l: u32,
    homo_m: i32,
    slot_period: u8,
    slot_group: u8,
}

#[wasm_bindgen]
impl ElementPresentationJs {
    #[wasm_bindgen(getter)]
    pub fn atomic_number(&self) -> u32 { self.atomic_number }
    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> String { self.symbol.to_string() }
    #[wasm_bindgen(getter)]
    pub fn display_name(&self) -> String { self.display_name.to_string() }
    #[wasm_bindgen(getter)]
    pub fn config_text(&self) -> String { self.config_text.to_string() }
    #[wasm_bindgen(getter)]
    pub fn homo_n(&self) -> u32 { self.homo_n }
    #[wasm_bindgen(getter)]
    pub fn homo_l(&self) -> u32 { self.homo_l }
    #[wasm_bindgen(getter)]
    pub fn homo_m(&self) -> i32 { self.homo_m }
    #[wasm_bindgen(getter)]
    pub fn slot_period(&self) -> u8 { self.slot_period }
    #[wasm_bindgen(getter)]
    pub fn slot_group(&self) -> u8 { self.slot_group }
}

/// Look up the shared element presentation for `element_z`.
///
/// Returns `None` for unsupported atomic numbers, matching the native
/// `atom_core::element_presentation` helper.
#[wasm_bindgen]
pub fn element_presentation(element_z: u32) -> Option<ElementPresentationJs> {
    let native = element::element_presentation(ElementId(element_z))?;
    Some(ElementPresentationJs {
        atomic_number: native.atomic_number,
        symbol: native.symbol,
        display_name: native.display_name,
        config_text: native.config_text,
        homo_n: native.homo.n,
        homo_l: native.homo.l,
        homo_m: native.homo.m,
        slot_period: native.slot.period,
        slot_group: native.slot.group,
    })
}

/// Decode a `v1:` URL string into a `DecodedScene`. Errors are surfaced
/// as `JsError` so JS-side `catch` clauses see a real `Error` with the
/// `DecodeError::Display` message, ready to drop into a banner.
#[wasm_bindgen]
pub fn scene_decode(s: &str) -> Result<DecodedScene, JsError> {
    let scene = scene::decode(s).map_err(|e| JsError::new(&format!("{}", e)))?;
    // decode is contractually single-atom at the origin — see scene.rs.
    let atom = &scene.atoms[0];
    Ok(DecodedScene {
        element_z: atom.element.0,
        n: atom.orbital.n,
        l: atom.orbital.l,
        m: atom.orbital.m,
        use_bare_z: scene.view.use_bare_z,
        colormap_id: scene.view.colormap.0,
        exposure: scene.view.exposure,
    })
}
