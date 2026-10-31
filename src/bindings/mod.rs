//! What every language binding shares: the Python module,
//! the wasm entry point, the number rules and the JavaScript brand.

#[cfg(python)]
pub mod python;

#[cfg(wasm)]
pub mod web;

#[cfg(any(wasm, mobile, python, test))]
pub(crate) mod numbers;

pub(crate) mod brand;
