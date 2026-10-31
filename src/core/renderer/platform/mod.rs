//! The renderer's binding shims, and the window checks each platform needs.

#[cfg(wasm)]
pub mod web;

#[cfg(python)]
mod python;
#[cfg(python)]
pub use python::*;

#[cfg(mobile)]
pub mod mobile;

#[cfg(any(android, test))]
pub(crate) mod android_window;

#[cfg(any(ios, all(test, macos)))]
pub(crate) mod metal_layer;

#[cfg(all(desktop, feature = "winit"))]
mod winit_window;
