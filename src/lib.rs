//! # FragmentColor
//!
//! Universal GPU abstraction for Javascript, Python, Swift, and Kotlin.

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
/// Higher-level objects that combine Mesh + Material into a renderable thing
/// with a per-instance transform. See [Model](crate::Model). A full Scene
/// object that owns many Models is on the roadmap.
pub mod scene;

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

/// # Networking module
///
/// Cross-target networking helpers (text/bytes over HTTP)
pub mod net;

/// # ScreenRegion module
///
/// ScreenRegion type conversions and collision detection
pub mod region;

/// Guides (developer docs)
///
/// Included as module so Rust examples run as doctests.
pub mod guides;

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

/// Macros for blanket type conversions
mod macros;

/// Top-level platform-specific initializers
mod platforms;

/// Stable kind branding across language bindings
pub mod fc_kind;

pub use {
    color::*, error::*, fc_kind::*, material::*, mesh::*, pass::*, region::*, renderer::*,
    scene::*, shader::*, size::*, target::*, texture::*,
};
