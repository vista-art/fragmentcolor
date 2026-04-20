#[cfg(python)]
pub mod python;

#[cfg(wasm)]
pub mod web;

/// uniffi bindings for iOS (Swift) and Android (Kotlin).
#[cfg(mobile)]
pub mod mobile;
