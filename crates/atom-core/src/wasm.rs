//! wasm-bindgen entry point for the volume bake.
//!
//! The browser-facing API is intentionally tiny. Slice 1 of the multi-atom
//! direction ships a Scene-shaped bake on the Rust side but only ever
//! instantiates a single hydrogen atom at the origin, so the JS↔WASM
//! signature is still the thin `(n, l, m, res)` tuple — just renamed
//! `bake_scene` to match the new core API and to mark the protocol as
//! Scene-shaped on the JS side. Adding multi-atom scenes to JS lands with
//! issue 02 / 05 (element picker + UI for arbitrary scenes).

use js_sys::Float32Array;
use wasm_bindgen::prelude::*;

use crate::scene::{Orbital, Scene};
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

/// Bake a Scene of a single hydrogen atom at the origin for orbital
/// (n, l, m) at the given grid resolution.
///
/// Mirrors `atom_core::volume::bake_scene` but constructs the Scene on the
/// Rust side so the JS↔WASM boundary stays a thin parameter tuple. Future
/// slices will widen this to accept a serialized Scene once the JS UI
/// supports multi-atom configurations.
#[wasm_bindgen]
pub fn bake_scene(n: u32, l: u32, m: i32, res: u32) -> BakeResult {
    let scene = Scene::single_hydrogen(Orbital { n, l, m });
    let v = volume::bake_scene(&scene, res);
    BakeResult {
        data: v.data,
        half_extent: v.half_extent as f32,
        peak: v.peak as f32,
    }
}
