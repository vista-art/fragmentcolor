mod python;
mod web;

#[cfg(python)]
pub use python::*;

#[cfg(wasm)]
pub use web::*;
