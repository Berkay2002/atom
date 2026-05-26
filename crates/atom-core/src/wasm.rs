//! wasm-bindgen entry point for the volume bake.
//!
//! The browser-facing API is intentionally tiny: a single `bake(n, l, m, res)`
//! function returns a `BakeResult` whose `data` getter exposes the f32 voxel
//! grid as a `Float32Array` view into wasm linear memory (no copy).
//!
//! The view is only valid as long as the underlying Vec lives on the Rust
//! side and isn't reallocated, so JS should copy out of it (or upload it
//! to a GPU texture) before dropping the `BakeResult`.

use js_sys::Float32Array;
use wasm_bindgen::prelude::*;

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

/// Bake the volume for orbital (n, l, m) at the given grid resolution.
///
/// Mirrors `atom_core::volume::bake` but returns a JS-friendly handle.
#[wasm_bindgen]
pub fn bake(n: u32, l: u32, m: i32, res: usize) -> BakeResult {
    let v = volume::bake(n, l, m, res);
    BakeResult {
        data: v.data,
        half_extent: v.half_extent as f32,
        peak: v.peak as f32,
    }
}
