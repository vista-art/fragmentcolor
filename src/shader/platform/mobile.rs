//! Mobile (Swift / Kotlin) uniffi bindings for `Shader`.
//!
//! The core `Shader::new(&str)` stays untouched for Rust users. Uniffi
//! always marshals strings by value, so the mobile constructor takes an
//! owned `String` and returns `Arc<Self>` as every uniffi object must.

use std::sync::Arc;

use lsp_doc::lsp_doc;

use crate::Shader;
use crate::renderer::platform::mobile::FragmentColorError;

#[uniffi::export]
impl Shader {
    /// Swift/Kotlin extensions re-expose this as `Shader(source)` for
    /// idiomatic construction that matches Rust, JavaScript and Python.
    #[uniffi::constructor]
    #[lsp_doc("docs/api/core/shader/hidden/new_mobile.md")]
    pub fn new_mobile(source: String) -> Result<Arc<Self>, FragmentColorError> {
        Shader::new(&source)
            .map(Arc::new)
            .map_err(FragmentColorError::from)
    }
}
