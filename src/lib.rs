//! # FragmentColor
//!
//! Easy GPU Rendering for Javascript, Python, Kotlin, and Swift.

#![allow(clippy::module_inception)]

#[cfg(not(wasm))]
uniffi::setup_scaffolding!();

/// # Renderer module.
///
/// This module contains the renderer and its related types.
/// Users do not need to use it directly.
///
/// A Global Renderer is lazily instanced by the App module
/// when the user creates the first Window or Web Canvas.
pub mod renderer;

/// # Shader Module
pub mod shader;

pub mod error;

// DRAFT
mod buffer_pool;
pub mod color;
pub mod frame;
pub mod pass;
pub mod region;
pub mod resources;
pub mod sampler;
pub mod target;
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
