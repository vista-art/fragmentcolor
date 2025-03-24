/// Shared initialization logic for all platforms.
pub(crate) mod all;
pub(crate) use all::*;

/// wasm-bindgen (Web)
#[cfg(wasm)]
pub mod web;

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

/// pyo3 (Python)
#[cfg(python)]
pub mod python;
#[cfg(python)]
pub use python::*;
