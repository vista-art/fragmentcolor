//! Declares the error shims.

#[cfg(mobile)]
pub(crate) mod mobile;

#[cfg(python)]
mod python;
#[cfg(python)]
pub use python::*;

#[cfg(wasm)]
mod web;
#[cfg(wasm)]
pub use web::*;
