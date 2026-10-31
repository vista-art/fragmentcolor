//! # FragmentColor
//!
//! Universal GPU abstraction for Rust, JavaScript, Python, Swift, and Kotlin.

// Uniffi scaffolding for Swift (iOS) and Kotlin (Android).
// Must be called at crate root. Compiled on every non-wasm target so
// `cargo run --bin uniffi-bindgen` works on desktop hosts — the generated
// FFI symbols are only *used* on mobile.
#[cfg(not(wasm))]
uniffi::setup_scaffolding!();

/// # Texture module
///
/// Texture represents an image stored on the GPU that can be sampled in shaders.
mod texture;

/// # Scene Module
///
/// Top-level container for the real-world things you render — Models,
/// Cameras, Lights, custom SceneObjects. See [Scene](crate::Scene),
/// [Model](crate::Model), [Camera](crate::Camera), and the unified
/// [Light](crate::Light) type with directional / point / spot
/// constructors.
mod scene;

/// # Error module
///
/// Common errors used across the library.
mod error;

/// Guides (developer docs)
///
/// Included as module so Rust examples run as doctests.
pub mod guides;

mod bindings;
mod conversion_macros;
mod core;
mod geometry;
mod network;
mod targets;
mod values;

#[cfg(test)]
mod testing;

#[cfg(all(not(wasm), feature = "winit"))]
mod winit_app;
#[cfg(all(not(wasm), feature = "winit"))]
pub use winit_app::*;

pub use self::core::*;
pub use error::*;
pub use geometry::*;
pub use network::NetworkError;
pub use scene::*;
pub use targets::*;
pub use texture::*;
pub use values::*;

pub(crate) use bindings::brand::{FcKind, impl_fc_kind};
#[cfg(wasm)]
pub(crate) use conversion_macros::impl_tryfrom_owned_via_ref;
pub(crate) use conversion_macros::{
    impl_from_into_with_refs, impl_js_bridge, impl_tryfrom_js_ref_anchor,
};
