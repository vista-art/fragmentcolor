//! Mobile (Swift / Kotlin) uniffi bindings for `Texture`.
//!
//! Mirrors the `wasm_bindgen` shim in `web.rs` and the `pyo3` shim in
//! `python.rs`: each foreign binding gets a thin wrapper around the
//! cross-platform `Texture` API. The Rust core method on `Texture` is
//! `set_sampler_options(opts: SamplerOptions)`; the mobile shim is named
//! `set_sampler_options_mobile` so the build-time doc scanner can keep
//! it separate from the Rust-only API, with the uniffi `name = "..."`
//! attribute exposing the idiomatic camelCase form.

use lsp_doc::lsp_doc;

use crate::{SamplerOptions, Texture};

#[uniffi::export]
impl Texture {
    /// Update the texture sampler options (filtering, wrapping, optional
    /// depth-compare). Mirrors the Web `setSamplerOptions` and Python
    /// `set_sampler_options` entry points; foreign bindings see this
    /// method as `setSamplerOptions(opts:)` (Swift) /
    /// `setSamplerOptions(opts)` (Kotlin) once the extension shims map
    /// the camelCase form back onto a single overload.
    #[uniffi::method(name = "setSamplerOptions")]
    #[lsp_doc("docs/api/core/texture/set_sampler_options.md")]
    pub fn set_sampler_options_mobile(&self, opts: SamplerOptions) {
        self.set_sampler_options(opts);
    }
}
