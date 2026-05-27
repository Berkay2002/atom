//! Parity smoke test for the wasm bindgen layer.
//!
//! Bakes a single-hydrogen scene (n=2, l=1, m=0, res=32) two ways on the
//! wasm32 target:
//!   1. Direct call to `atom_core::volume::bake_scene`.
//!   2. Through the `#[wasm_bindgen]` `atom_core::wasm::bake_scene` entry
//!      point, reading back the Float32Array view.
//!
//! Asserts the two slices are equal element-wise within 1e-6. This proves
//! the bindgen layer (Float32Array view, BakeResult getters) doesn't
//! corrupt or copy-mangle the voxel data.
//!
//! Run with: wasm-pack test --node crates/atom-core --features wasm

#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen_test::*;

// `wasm-pack test --node` runs in Node by default; no `wasm_bindgen_test_configure!`
// needed (and 0.3.72+ dropped the `run_in_node` directive).

#[wasm_bindgen_test]
fn wasm_bake_matches_native_bake_elementwise() {
    let (n, l, m, res) = (2u32, 1u32, 0i32, 32u32);

    let scene = atom_core::scene::Scene::single_hydrogen(atom_core::scene::Orbital { n, l, m });
    let native = atom_core::volume::bake_scene(&scene, res);
    // Element Z=1 (hydrogen), use_bare_z=false → identical to single_hydrogen.
    let wasm = atom_core::wasm::bake_scene(1, n, l, m, false, res);

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
            "voxel {i} differs: native={a} wasm={b}"
        );
    }

    // Sanity-check the scalar fields too.
    assert!(
        (wasm.half_extent() as f64 - native.half_extent).abs() < 1e-6,
        "half_extent mismatch"
    );
    assert!(
        (wasm.peak() as f64 - native.peak).abs() < 1e-3 * native.peak,
        "peak mismatch"
    );
}

#[wasm_bindgen_test]
fn wasm_element_presentation_matches_native_projection() {
    for z in 1u32..=18 {
        let native = atom_core::element::element_presentation(atom_core::scene::ElementId(z))
            .expect("supported element");
        let wasm = atom_core::wasm::element_presentation(z).expect("supported element");

        assert_eq!(wasm.atomic_number(), native.atomic_number, "Z={z} atomic number");
        assert_eq!(wasm.symbol(), native.symbol, "Z={z} symbol");
        assert_eq!(wasm.display_name(), native.display_name, "Z={z} display name");
        assert_eq!(wasm.config_text(), native.config_str, "Z={z} config text");
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
