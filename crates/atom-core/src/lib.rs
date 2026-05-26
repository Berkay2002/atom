//! Shared hydrogen wavefunction math and volume bake.
//!
//! This crate contains the platform-independent core of the atom visualizer:
//! the closed-form `|ψ_nlm|²` evaluator and the rayon-parallel voxel bake.
//! It compiles to native rlib for `atom-desktop` and (in a later slice) to
//! cdylib for the WebAssembly web demo.

pub mod physics;
pub mod volume;
