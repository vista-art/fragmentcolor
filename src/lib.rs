//! # FragmentColor
//!
//! Easy GPU Rendering for Javascript, Python, Kotlin, and Swift.

#![allow(clippy::module_inception)]

/// # The Components Collection.
///
/// This is the module that users will play with more frequently.
///
///
///
/// Typically, users do not need to use the components directly,
/// but rather use the `Object` struct, which is a wrapper
/// around spatial components which can be added to a Scene.
///
/// Types of Components:
/// - Object (spatial components)
///     - Sprite
///     - Shape
///     - Mesh
///
/// - Marker components
pub mod components;

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

/// # Resources Module
pub mod resources;

/// # Scene Graph Module
pub mod scene;

pub use components::*;
pub use math::*;
pub use renderer::*;
pub use resources::*;
pub use scene::*;
