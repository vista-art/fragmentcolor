//! # FragmentColor
//!
//! Easy GPU Rendering for Javascript, Python, Kotlin, and Swift.

#![allow(clippy::module_inception)]

#[cfg(not(wasm))]
uniffi::setup_scaffolding!();

#[cfg_attr(feature = "python", pyo3::pyclass)]
pub struct FragmentColor;

/// # Platform-specific implementations
pub mod platform;
pub use platform::*;

/// # Renderer module.
///
pub mod renderer;

/// # Shader Module
///
pub mod shader;

/// # Error module
pub mod error;

// DRAFT; may change in a whim
mod buffer_pool;
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
pub use texture::*;
