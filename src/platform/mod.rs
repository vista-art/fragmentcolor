pub mod all;

#[cfg(wasm)]
pub mod web;

#[cfg(android)]
pub mod android;

#[cfg(ios)]
pub mod ios;

#[cfg(not(any(wasm, android, ios)))]
pub mod desktop;

pub mod target;
pub use target::*;
