//! wasm-bindgen entry point for the volume bake.
//!
//! The browser-facing API is intentionally tiny. The single-atom shape
//! still flows as a thin parameter tuple — `(element_z, n, l, m,
//! use_bare_z, res)` — because the JS UI only configures one atom today.
//! Multi-atom scenes will widen this once issue 05 (richer scene UI)
//! lands.

use js_sys::Float32Array;
use wasm_bindgen::prelude::*;

use crate::scene::{Atom, ElementId, Orbital, Scene, View};
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
