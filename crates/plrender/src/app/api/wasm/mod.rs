#![cfg(wasm)]

#[cfg(not(wasm))]
compile_error!("This module only supports Wasm target!");

pub mod canvas;

pub use canvas::*;
