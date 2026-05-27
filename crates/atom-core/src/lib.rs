//! Shared hydrogen wavefunction math and volume bake.
//!
//! This crate contains the platform-independent core of the atom visualizer:
//! the closed-form `|ψ_nlm|²` evaluator and the rayon-parallel voxel bake.
//! It compiles to native rlib for `atom-desktop` and (in a later slice) to
//! cdylib for the WebAssembly web demo.

pub mod element;
pub mod physics;
pub mod scene;
pub mod slater;
pub mod volume;

pub use element::{caption, element_data, orbital_description, orbital_label, ElementData, ELEMENTS};
pub use scene::{
    decode as scene_decode, encode as scene_encode, Atom, CameraState, ColormapId, DecodeError,
    ElementId, EncodeError, Orbital, Scene, View,
};
pub use slater::z_eff;

#[cfg(feature = "wasm")]
pub mod wasm;
