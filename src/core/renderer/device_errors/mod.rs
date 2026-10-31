//! The GPU errors the device reports, from the error scopes GPU work runs in.

#[cfg(any(wasm, test))]
pub(super) mod late;
pub(super) mod scopes;
