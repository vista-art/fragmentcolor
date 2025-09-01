//! # FragmentColor
//!
//! Easy GPU Rendering for Javascript, Python, Kotlin, and Swift.

#[cfg(not(wasm))]
uniffi::setup_scaffolding!();

/// # Renderer module.
///
pub mod renderer;

/// # Target module
///
pub mod target;

/// # Shader Module
///
pub mod shader;

/// # Pass Module
///
pub mod pass;

/// # Frame module
///
pub mod frame;

/// # Error module
///
pub mod error;

// DRAFT; API may change in a whim
pub mod color;
pub mod region;
pub mod sampler;
pub mod size;

pub use {
    color::*, error::*, frame::*, pass::*, region::*, renderer::*, sampler::*, shader::*, size::*,
    target::*,
};
