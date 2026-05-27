//! Parity smoke test for the wasm bindgen layer.
//!
//! Bakes a single-hydrogen scene (n=2, l=1, m=0, res=32) two ways on the
//! wasm32 target:
//!   1. Direct call to `atom_core::volume::bake_scene`.
//!   2. Through the `#[wasm_bindgen]` projection-shaped
//!      `atom_core::wasm::bake_scene_projection` entry point, reading back
//!      the Float32Array view.
//!
//! Asserts the two slices are equal element-wise within 1e-6. This proves
//! the bindgen layer (Float32Array view, BakeResult getters) doesn't
//! corrupt or copy-mangle the voxel data.
//!
//! Run with: wasm-pack test --node crates/atom-core --features wasm

#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen_test::*;

use atom_core::scene::{Atom, ElementId, Orbital, Scene, View};

// `wasm-pack test --node` runs in Node by default; no `wasm_bindgen_test_configure!`
// needed (and 0.3.72+ dropped the `run_in_node` directive).

fn browser_projection(
    element_z: u32,
    n: u32,
    l: u32,
    m: i32,
    use_bare_z: bool,
) -> wasm_bindgen::JsValue {
    let projection = js_sys::Object::new();
    js_sys::Reflect::set(
        &projection,
        &wasm_bindgen::JsValue::from_str("elementZ"),
        &wasm_bindgen::JsValue::from_f64(element_z as f64),
    )
    .unwrap();
    js_sys::Reflect::set(
        &projection,
        &wasm_bindgen::JsValue::from_str("n"),
        &wasm_bindgen::JsValue::from_f64(n as f64),
    )
    .unwrap();
    js_sys::Reflect::set(
        &projection,
        &wasm_bindgen::JsValue::from_str("l"),
        &wasm_bindgen::JsValue::from_f64(l as f64),
    )
    .unwrap();
    js_sys::Reflect::set(
        &projection,
        &wasm_bindgen::JsValue::from_str("m"),
        &wasm_bindgen::JsValue::from_f64(m as f64),
    )
    .unwrap();
    js_sys::Reflect::set(
        &projection,
        &wasm_bindgen::JsValue::from_str("useBareZ"),
        &wasm_bindgen::JsValue::from_bool(use_bare_z),
    )
    .unwrap();
    js_sys::Reflect::set(
        &projection,
        &wasm_bindgen::JsValue::from_str("colormapId"),
        &wasm_bindgen::JsValue::from_f64(0.0),
    )
    .unwrap();
    js_sys::Reflect::set(
        &projection,
        &wasm_bindgen::JsValue::from_str("exposure"),
        &wasm_bindgen::JsValue::from_f64(1.0),
    )
    .unwrap();
    projection.into()
}

fn single_atom_scene(element_z: u32, n: u32, l: u32, m: i32, use_bare_z: bool) -> Scene {
    Scene {
        atoms: vec![Atom {
            element: ElementId(element_z),
            position: [0.0, 0.0, 0.0],
            orbital: Orbital { n, l, m },
        }],
        view: View {
            use_bare_z,
            ..View::default()
        },
    }
}

fn assert_projection_bake_matches_native(
    element_z: u32,
    n: u32,
    l: u32,
    m: i32,
    use_bare_z: bool,
    res: u32,
) {
    let scene = single_atom_scene(element_z, n, l, m, use_bare_z);
    let native = atom_core::volume::bake_scene(&scene, res as usize);
    let wasm = atom_core::wasm::bake_scene_projection(
        browser_projection(element_z, n, l, m, use_bare_z),
        res,
    )
    .expect("projection bake should accept a codec-shaped browser projection");

    // Pull the Float32Array view across the JS boundary into a Rust Vec.
    let view = wasm.data();
    let len = view.length() as usize;
    let res = res as usize;
    assert_eq!(len, res * res * res, "data length mismatch");
    assert_eq!(len, native.data.len(), "native length mismatch");

    let mut roundtrip = vec![0.0f32; len];
    view.copy_to(&mut roundtrip);

    // Element-wise parity. Both runs share the same wasm32 codegen, so we
    // expect bit-identical results, but the contract is 1e-6.
    for (i, (&a, &b)) in native.data.iter().zip(roundtrip.iter()).enumerate() {
        assert!(
            (a - b).abs() < 1e-6,
            "Z={element_z} bare={use_bare_z} voxel {i} differs: native={a} wasm={b}"
        );
    }

    // Sanity-check the scalar fields too.
    assert!(
        (wasm.half_extent() as f64 - native.half_extent).abs() < 1e-6,
        "Z={element_z} bare={use_bare_z} half_extent mismatch"
    );
    assert!(
        (wasm.peak() as f64 - native.peak).abs() < 1e-3 * native.peak,
        "Z={element_z} bare={use_bare_z} peak mismatch"
    );
}

#[wasm_bindgen_test]
fn wasm_projection_bake_matches_native_bake_elementwise() {
    let cases = [
        // Hydrogen, effective-Z path: the original single_hydrogen baseline.
        (1u32, 2u32, 1u32, 0i32, false),
        // Carbon, effective-Z path: proves elementZ crosses the projection.
        (6u32, 2u32, 1u32, 0i32, false),
        // Carbon, bare-Z path: proves useBareZ crosses the projection.
        (6u32, 2u32, 1u32, 0i32, true),
    ];

    for (element_z, n, l, m, use_bare_z) in cases {
        assert_projection_bake_matches_native(element_z, n, l, m, use_bare_z, 32);
    }
}

#[wasm_bindgen_test]
fn wasm_projection_caption_matches_native_caption() {
    let cases = [
        (1u32, 1u32, 0u32, 0i32, false),
        (6u32, 2u32, 1u32, 1i32, false),
        (10u32, 2u32, 1u32, 0i32, true),
        (99u32, 4u32, 3u32, 0i32, false),
    ];

    for (element_z, n, l, m, use_bare_z) in cases {
        let scene = single_atom_scene(element_z, n, l, m, use_bare_z);
        let native = atom_core::element::caption(&scene);
        let wasm = atom_core::wasm::scene_caption_projection(browser_projection(
            element_z, n, l, m, use_bare_z,
        ))
        .expect("projection caption should accept a codec-shaped browser projection");

        assert_eq!(wasm, native, "Z={element_z} n={n} l={l} m={m}");
    }
}

#[wasm_bindgen_test]
fn wasm_element_presentation_matches_native_projection() {
    for z in 1u32..=18 {
        let native = atom_core::element::element_presentation(atom_core::scene::ElementId(z))
            .expect("supported element");
        let wasm = atom_core::wasm::element_presentation(z).expect("supported element");

        assert_eq!(
            wasm.atomic_number(),
            native.atomic_number,
            "Z={z} atomic number"
        );
        assert_eq!(wasm.symbol(), native.symbol, "Z={z} symbol");
        assert_eq!(
            wasm.display_name(),
            native.display_name,
            "Z={z} display name"
        );
        assert_eq!(wasm.config_text(), native.config_text, "Z={z} config text");
        assert_eq!(wasm.homo_n(), native.homo.n, "Z={z} homo n");
        assert_eq!(wasm.homo_l(), native.homo.l, "Z={z} homo l");
        assert_eq!(wasm.homo_m(), native.homo.m, "Z={z} homo m");
        assert_eq!(wasm.slot_period(), native.slot.period, "Z={z} slot period");
        assert_eq!(wasm.slot_group(), native.slot.group, "Z={z} slot group");
    }

    assert!(atom_core::wasm::element_presentation(0).is_none());
    assert!(atom_core::wasm::element_presentation(19).is_none());
    assert!(atom_core::wasm::element_presentation(99).is_none());
}
