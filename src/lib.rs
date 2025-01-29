//! # FragmentColor
//!
//! Easy GPU Rendering for Javascript, Python, Kotlin, and Swift.

#![allow(clippy::module_inception)]

#[cfg(not(wasm))]
uniffi::setup_scaffolding!();

/// # Math Module
///
/// This module contains the math types and functions used by the library.
pub mod math;

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

pub mod uniform;

/// # Shader Errors
pub mod error;

// DRAFT
pub mod color;
pub mod frame;
pub mod pass;

pub use color::*;
pub use error::*;
pub use frame::*;
pub use math::*;
pub use pass::*;
pub use renderer::*;
pub use shader::*;
pub use uniform::*;
