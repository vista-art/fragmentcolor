//! Renderer module.
//!
//! This module contains the renderer and its related types.
//! Users do not need to use this module directly.
//!
//! A Global Renderer is lazily instanced by the App module
//! when the user creates the first Window or Web Canvas.

mod limits;
pub mod options;
pub mod renderer;
pub mod renderpass;
pub mod target;

pub use options::*;
pub use renderer::*;
pub use renderpass::*;
pub use target::*;
