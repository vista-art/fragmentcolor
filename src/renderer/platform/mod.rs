/// Shared initialization logic for all platforms.
pub(crate) mod all;

/// wasm-bindgen (Web)
#[cfg(wasm)]
pub mod web;

/// pyo3 (Python)
#[cfg(python)]
pub mod python;
#[cfg(python)]
pub use python::*;

/// uniffi (iOS + Android).
#[cfg(mobile)]
pub mod mobile;

/// Desktop window integration (winit)
#[cfg(all(desktop, feature = "winit"))]
pub mod winit;
