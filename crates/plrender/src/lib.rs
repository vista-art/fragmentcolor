//! Multiplatform GPU Rendering API for Javascript, Python and Beyond

/// # PLRender Application Module
///
/// This module contains the main App struct and its related types.
///
/// The main App instance is a singleton responsible for managing the global
/// resources of this Library, namely the main Event Loop and the Renderer.
///
/// It also contains internal utility functions, build metadata, errors, and
/// the Window/Canvas object to create and manage windows.
pub mod app;

/// # The Components Collection.
/// 
/// This is the module that users will play with more frequently.
///
/// 
/// 
/// Typically, users do not need to use the components directly,
/// but rather use the `SceneObject` struct, which is a wrapper
/// around spatial components which can be added to a Scene.
///
/// Types of Components:
/// - SceneObject (spatial components)
///     - Sprite
///     - Shape
///     - Mesh
///
/// - Marker components
pub mod components;


pub mod math;
pub mod renderer;
pub mod resources;
pub mod scene;

pub use app::*;
pub use components::*;
pub use math::*;
pub use renderer::*;
pub use resources::*;
pub use scene::*;
