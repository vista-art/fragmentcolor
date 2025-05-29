//! # FragmentColor
//!
//! Easy GPU Rendering for Javascript, Python, Kotlin, and Swift.

#[cfg(not(wasm))]
uniffi::setup_scaffolding!();

#[cfg_attr(feature = "python", pyo3::pyclass)]
pub struct FragmentColor;

/// # Renderer module.
///
pub mod renderer;

/// # Shader Module
///
pub mod shader;

/// # Error module
pub mod error;

/// # Target module
pub mod target;

// DRAFT; may change in a whim
pub mod color;
pub mod frame;
pub mod pass;
pub mod region;
pub mod resources;
pub mod sampler;
pub mod texture;

pub use color::*;
pub use error::*;
pub use frame::*;
pub use pass::*;
pub use region::*;
pub use renderer::*;
pub use resources::*;
pub use sampler::*;
pub use shader::*;
pub use target::*;
pub use texture::*;
