//! # FragmentColor
//!
//! Easy GPU Rendering for Javascript, Python, Kotlin, and Swift.

#[cfg(not(wasm))]
uniffi::setup_scaffolding!();

/// # Renderer module.
pub mod renderer;

/// # Shader Module
pub mod shader;

/// # Pass module
pub mod pass;

/// # Frame module
pub mod frame;

/// # Target module
pub mod target;

/// # Error module
pub mod error;

// DRAFT; may change in a whim
pub mod color;
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
