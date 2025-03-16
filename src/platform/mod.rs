/// Shared initialization logic for all platforms.
pub mod all;

/// Trait for a target that can be rendered to.
/// Must be implemented for each platform.
pub mod target;
pub use target::*;

/// Target implementation for Textures.
/// (shared by all platforms)
pub mod texture;
pub use texture::*;

/// wasm-bindgen (Web)
#[cfg(wasm)]
pub mod web;
#[cfg(wasm)]
pub use web::*;

/// uniffi (Android)
#[cfg(android)]
pub mod android;
#[cfg(android)]
pub use android::*;

/// uniffi (iOS)
#[cfg(ios)]
pub mod ios;
#[cfg(ios)]
pub use ios::*;

/// winit (Rust)
#[cfg(desktop)]
pub mod winit;
#[cfg(desktop)]
pub use winit::*;

/// pyo3 (Python)
#[cfg(python)]
pub mod python;
#[cfg(python)]
pub use python::*;
