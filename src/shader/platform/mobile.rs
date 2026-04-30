//! Mobile (Swift / Kotlin) uniffi bindings for `Shader`.
//!
//! Uniffi cannot express `impl Into<ShaderInput>` over the FFI boundary, so we
//! expose two named constructors: `new(source)` for the single-string form and
//! `compose(parts)` for the array form. Each input runs through the same
//! classifier (raw source / slug / URL / path) on the Rust side. Swift / Kotlin
//! extension shims provide a single overloaded `Shader(_:)` that picks the
//! right constructor based on argument type.

use std::sync::Arc;

use lsp_doc::lsp_doc;

use crate::Shader;
use crate::renderer::platform::mobile::FragmentColorError;

#[uniffi::export]
impl Shader {
    /// Foreign bindings see this as `Shader.new(source)`. On Swift, uniffi
    /// generates a `convenience init` when the constructor name is `new`,
    /// so callers write `let shader = try Shader(source: "...")`. Kotlin
    /// keeps the same factory signature.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/core/shader/hidden/new_mobile.md")]
    pub fn new_mobile(source: String) -> Result<Arc<Self>, FragmentColorError> {
        Shader::new(source)
            .map(Arc::new)
            .map_err(FragmentColorError::from)
    }

    /// Compose a shader from an array of parts (raw sources, slugs, URLs, paths).
    /// Each entry is classified independently; results are deduplicated and
    /// concatenated in order.
    #[uniffi::constructor(name = "compose")]
    #[lsp_doc("docs/api/core/shader/hidden/compose_mobile.md")]
    pub fn compose_mobile(parts: Vec<String>) -> Result<Arc<Self>, FragmentColorError> {
        Shader::new(parts)
            .map(Arc::new)
            .map_err(FragmentColorError::from)
    }
}

/// Override the slug registry base URL (process-wide).
///
/// Surfaces in Swift / Kotlin as a top-level function `setShaderRegistry(baseUrl:)`.
#[uniffi::export]
pub fn set_shader_registry(base_url: String) {
    Shader::set_registry(&base_url);
}
