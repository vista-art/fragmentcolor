/// Shared initialization logic for all platforms.
pub mod all;
pub use all::*;

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

/// pyo3 (Python)
#[cfg(python)]
pub mod python;
#[cfg(python)]
pub use python::*;

/// Desktop window integration (winit)
#[cfg(all(desktop, feature = "winit"))]
pub mod winit;
