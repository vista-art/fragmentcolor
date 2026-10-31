//! # FragmentColor
//!
//! Universal GPU abstraction for Rust, JavaScript, Python, Swift, and Kotlin.

// uniffi's scaffolding for Swift and Kotlin has to sit at the crate root.
// It compiles on every target but wasm, so `cargo run --bin uniffi-bindgen`
// works on a desktop host, though only the mobile builds use its FFI symbols.
#[cfg(not(wasm))]
uniffi::setup_scaffolding!();

pub mod guides;

mod bindings;
mod conversion_macros;
mod core;
mod error;
mod geometry;
mod network;
mod scene;
mod targets;
mod texture;
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

pub(crate) use bindings::brand::{
    FcKind, impl_fc_kind, impl_js_bridge, impl_tryfrom_js_ref_anchor,
};
pub(crate) use conversion_macros::impl_from_into_with_refs;
#[cfg(wasm)]
pub(crate) use conversion_macros::impl_tryfrom_owned_via_ref;
