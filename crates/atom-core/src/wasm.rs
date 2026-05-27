//! wasm-bindgen entry point for the volume bake.
//!
//! The browser-facing API is intentionally tiny. Scene-shaped operations
//! cross the JS/WASM boundary through the same single-atom projection
//! object, with bake resolution passed separately because it is a render
//! quality choice rather than part of the Scene.

use js_sys::{Float32Array, Object, Reflect};
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

fn bake_result(scene: &Scene, res: u32) -> BakeResult {
    let v = volume::bake_scene(scene, res as usize);
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
// the JS side hands us one narrow object with the fields the URL carries,
// and decode returns that same object shape.

const ELEMENT_Z: &str = "elementZ";
const N: &str = "n";
const L: &str = "l";
const M: &str = "m";
const USE_BARE_Z: &str = "useBareZ";
const COLORMAP_ID: &str = "colormapId";
const EXPOSURE: &str = "exposure";

fn projection_field(projection: &JsValue, field: &str) -> Result<JsValue, JsError> {
    Reflect::get(projection, &JsValue::from_str(field)).map_err(|_| {
        JsError::new(&format!(
            "invalid browser scene projection: missing field '{field}'"
        ))
    })
}

fn projection_u32(projection: &JsValue, field: &str) -> Result<u32, JsError> {
    let value = projection_field(projection, field)?;
    let Some(number) = value.as_f64() else {
        return Err(JsError::new(&format!(
            "invalid browser scene projection: field '{field}' must be a number"
        )));
    };
    if !number.is_finite() || number.fract() != 0.0 || number < 0.0 || number > u32::MAX as f64 {
        return Err(JsError::new(&format!(
            "invalid browser scene projection: field '{field}' must be a finite unsigned integer"
        )));
    }
    Ok(number as u32)
}

fn projection_i32(projection: &JsValue, field: &str) -> Result<i32, JsError> {
    let value = projection_field(projection, field)?;
    let Some(number) = value.as_f64() else {
        return Err(JsError::new(&format!(
            "invalid browser scene projection: field '{field}' must be a number"
        )));
    };
    if !number.is_finite()
        || number.fract() != 0.0
        || number < i32::MIN as f64
        || number > i32::MAX as f64
    {
        return Err(JsError::new(&format!(
            "invalid browser scene projection: field '{field}' must be a finite signed integer"
        )));
    }
    Ok(number as i32)
}

fn projection_f32(projection: &JsValue, field: &str) -> Result<f32, JsError> {
    let value = projection_field(projection, field)?;
    let Some(number) = value.as_f64() else {
        return Err(JsError::new(&format!(
            "invalid browser scene projection: field '{field}' must be a number"
        )));
    };
    if !number.is_finite() {
        return Err(JsError::new(&format!(
            "invalid browser scene projection: field '{field}' must be finite"
        )));
    }
    Ok(number as f32)
}

fn projection_bool(projection: &JsValue, field: &str) -> Result<bool, JsError> {
    let value = projection_field(projection, field)?;
    value.as_bool().ok_or_else(|| {
        JsError::new(&format!(
            "invalid browser scene projection: field '{field}' must be a boolean"
        ))
    })
}

fn scene_from_projection(projection: &JsValue) -> Result<Scene, JsError> {
    Ok(Scene {
        atoms: vec![Atom {
            element: ElementId(projection_u32(projection, ELEMENT_Z)?),
            position: [0.0, 0.0, 0.0],
            orbital: Orbital {
                n: projection_u32(projection, N)?,
                l: projection_u32(projection, L)?,
                m: projection_i32(projection, M)?,
            },
        }],
        view: View {
            use_bare_z: projection_bool(projection, USE_BARE_Z)?,
            camera: crate::scene::CameraState::default(),
            colormap: crate::scene::ColormapId(projection_u32(projection, COLORMAP_ID)?),
            exposure: projection_f32(projection, EXPOSURE)?,
        },
    })
}

fn set_projection_field(object: &Object, field: &str, value: JsValue) {
    Reflect::set(object, &JsValue::from_str(field), &value)
        .expect("setting a plain object property should succeed");
}

fn projection_from_scene(scene: &Scene) -> JsValue {
    let atom = &scene.atoms[0];
    let object = Object::new();
    set_projection_field(&object, ELEMENT_Z, JsValue::from_f64(atom.element.0 as f64));
    set_projection_field(&object, N, JsValue::from_f64(atom.orbital.n as f64));
    set_projection_field(&object, L, JsValue::from_f64(atom.orbital.l as f64));
    set_projection_field(&object, M, JsValue::from_f64(atom.orbital.m as f64));
    set_projection_field(
        &object,
        USE_BARE_Z,
        JsValue::from_bool(scene.view.use_bare_z),
    );
    set_projection_field(
        &object,
        COLORMAP_ID,
        JsValue::from_f64(scene.view.colormap.0 as f64),
    );
    set_projection_field(
        &object,
        EXPOSURE,
        JsValue::from_f64(scene.view.exposure as f64),
    );
    object.into()
}

/// Encode the slice-1 single-atom browser projection to a `v1:` URL string.
#[wasm_bindgen]
pub fn scene_encode_projection(projection: JsValue) -> Result<String, JsError> {
    let scene = scene_from_projection(&projection)?;
    scene::encode(&scene).map_err(|e| JsError::new(&format!("{}", e)))
}

/// Decode a `v1:` URL string into the browser projection object shape.
#[wasm_bindgen]
pub fn scene_decode_projection(s: &str) -> Result<JsValue, JsError> {
    let scene = scene::decode(s).map_err(|e| JsError::new(&format!("{}", e)))?;
    Ok(projection_from_scene(&scene))
}

/// Bake the slice-1 single-atom browser projection at the given grid
/// resolution.
///
/// `projection` uses the same object shape as `scene_encode_projection` and
/// `scene_decode_projection`: element, orbital, and view fields are Scene
/// facts, while `res` stays an explicit bake quality parameter. The returned
/// data view has the same ownership contract as `BakeResult::data`: JS must
/// copy or upload it before freeing this result.
#[wasm_bindgen]
pub fn bake_scene_projection(projection: JsValue, res: u32) -> Result<BakeResult, JsError> {
    let scene = scene_from_projection(&projection)?;
    Ok(bake_result(&scene, res))
}

/// Compose the user-facing caption for the slice-1 single-atom browser
/// projection.
///
/// The returned string is the same line `atom_core::caption(&scene)`
/// produces for the equivalent single-atom `Scene`.
#[wasm_bindgen]
pub fn scene_caption_projection(projection: JsValue) -> Result<String, JsError> {
    let scene = scene_from_projection(&projection)?;
    Ok(crate::element::caption(&scene))
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
    pub fn atomic_number(&self) -> u32 {
        self.atomic_number
    }
    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> String {
        self.symbol.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn display_name(&self) -> String {
        self.display_name.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn config_text(&self) -> String {
        self.config_text.to_string()
    }
    #[wasm_bindgen(getter)]
    pub fn homo_n(&self) -> u32 {
        self.homo_n
    }
    #[wasm_bindgen(getter)]
    pub fn homo_l(&self) -> u32 {
        self.homo_l
    }
    #[wasm_bindgen(getter)]
    pub fn homo_m(&self) -> i32 {
        self.homo_m
    }
    #[wasm_bindgen(getter)]
    pub fn slot_period(&self) -> u8 {
        self.slot_period
    }
    #[wasm_bindgen(getter)]
    pub fn slot_group(&self) -> u8 {
        self.slot_group
    }
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
