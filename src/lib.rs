//! # FragmentColor
//!
//! Easy GPU Rendering for Javascript, Python, Kotlin, and Swift.

// To be enabled in Version 0.10.8 (for Android and iOS support)
//
// #[cfg(not(wasm))]
// uniffi::setup_scaffolding!();

/// # Renderer module.
///
/// The Renderer is the main entry point for rendering operations.
pub mod renderer;

/// # Target module
///
/// Target represents a rendering destination, such as a window or an offscreen texture.
pub mod target;

/// # Texture module
///
/// Texture represents an image stored on the GPU that can be sampled in shaders.
pub mod texture;

/// # Shader Module
///
/// Shader represents a GPU program that can be used in Passes to render graphics.
pub mod shader;

/// # Pass Module
///
/// A Pass represents a single rendering operation that can be part of a Frame.
pub mod pass;

/// # Frame module
///
/// Frame represents a collection of Passes that can be rendered together.
pub mod frame;

/// # Error module
///
/// Common errors used across the library.
pub mod error;

/// # Size module
///
/// Simple helper to convert between different size representations.
pub mod size;

/// # Color module
///
/// Simple helper to represent color and convert user input (i.e. CSS strings)
pub mod color;

/// Cross-target networking helpers (text/bytes over HTTP)
pub mod net;

/// Winit App Module (desktop only)
///
/// Requires the `winit` feature to be enabled.
///
/// Simple convenience wrapper around winit to simplify our Rust examples.
/// Implements winit's ApplicationHandler and contains all FragmentColor objects.
#[cfg(all(not(wasm), feature = "winit"))]
pub mod app;
#[cfg(all(not(wasm), feature = "winit"))]
pub use app::*;

/// DRAFT; API may change in a whim
pub mod region;

// Macros for blanket type conversions
mod macros;

pub use {
    color::*, error::*, frame::*, pass::*, region::*, renderer::*, shader::*, size::*, target::*,
    texture::*,
};

/// Install a panic hook and console logger when running in WASM so browser console shows
/// readable errors instead of a generic "unreachable" trap.
#[cfg(wasm)]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);
}
