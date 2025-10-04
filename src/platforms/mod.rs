#[cfg(python)]
pub mod python;

// To be enabled in Version 0.10.8 (for Android and iOS support)
//
// #[cfg(not(wasm))]
// uniffi::setup_scaffolding!();

#[cfg(wasm)]
pub mod web;
