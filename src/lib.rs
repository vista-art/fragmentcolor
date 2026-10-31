//! # FragmentColor
//!
//! Universal GPU abstraction for Rust, JavaScript, Python, Swift, and Kotlin.

// Uniffi scaffolding for Swift (iOS) and Kotlin (Android).
// Must be called at crate root. Compiled on every non-wasm target so
// `cargo run --bin uniffi-bindgen` works on desktop hosts — the generated
// FFI symbols are only *used* on mobile.
#[cfg(not(wasm))]
uniffi::setup_scaffolding!();

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
/// A Pass represents a single rendering operation; any iterable of Pass is renderable.
pub mod pass;

/// # Mesh Module
///
/// Mesh represents geometry (vertices, indices, instances) and can be attached to a Pass.
pub mod mesh;

/// # Material Module
///
/// Material bundles PBR factors / textures with the Shader that knows how to
/// shade them. See [Material::pbr](crate::Material::pbr) for the built-in
/// physically-based default.
pub mod material;

/// # Scene Module
///
/// Top-level container for the real-world things you render — Models,
/// Cameras, Lights, custom SceneObjects. See [Scene](crate::Scene),
/// [Model](crate::Model), [Camera](crate::Camera), and the unified
/// [Light](crate::Light) type with directional / point / spot
/// constructors.
pub mod scene;

/// # Error module
///
/// Common errors used across the library.
#[expect(
    hidden_glob_reexports,
    reason = "mesh, pass, renderer, shader and target glob-export a public `error` module this one hides"
)]
mod error;

/// Guides (developer docs)
///
/// Included as module so Rust examples run as doctests.
pub mod guides;

mod bindings;
mod conversion_macros;
mod network;
mod values;

#[cfg(test)]
mod testing;

#[cfg(all(not(wasm), feature = "winit"))]
mod winit_app;
#[cfg(all(not(wasm), feature = "winit"))]
pub use winit_app::*;

pub use error::*;
pub use material::*;
pub use mesh::*;
pub use network::NetworkError;
pub use pass::*;
pub use renderer::*;
pub use scene::*;
pub use shader::*;
pub use target::*;
pub use texture::*;
pub use values::*;

pub(crate) use bindings::brand::{FcKind, impl_fc_kind};
#[cfg(wasm)]
pub(crate) use conversion_macros::impl_tryfrom_owned_via_ref;
pub(crate) use conversion_macros::{
    impl_from_into_with_refs, impl_js_bridge, impl_tryfrom_js_ref_anchor,
};
