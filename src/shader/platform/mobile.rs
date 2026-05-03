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
use crate::shader::uniform::UniformData;

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

    #[uniffi::constructor(name = "default")]
    #[lsp_doc("docs/api/core/shader/default.md")]
    pub fn default_mobile() -> Arc<Self> {
        Arc::new(Shader::default())
    }

    /// Override the slug registry base URL (process-wide).
    /// Uniffi has no static-method form on `uniffi::Object`, so this is
    /// expressed as a constructor that performs the side effect and returns
    /// a default `Shader`. The Swift / Kotlin extension shims discard the
    /// returned instance so callers see `Shader.setRegistry(baseUrl:)` with
    /// `Void` return — matching the JS / Python static-method spelling.
    #[uniffi::constructor(name = "setRegistry")]
    #[lsp_doc("docs/api/core/shader/set_registry.md")]
    pub fn set_registry_mobile(base_url: String) -> Arc<Self> {
        Shader::set_registry(&base_url);
        Arc::new(Shader::default())
    }

    #[uniffi::method(name = "set")]
    #[lsp_doc("docs/api/core/shader/set.md")]
    pub fn set_mobile(&self, key: String, value: UniformData) -> Result<(), FragmentColorError> {
        self.set(&key, value).map_err(FragmentColorError::from)
    }

    #[uniffi::method(name = "get")]
    #[lsp_doc("docs/api/core/shader/get.md")]
    pub fn get_mobile(&self, key: String) -> Result<UniformData, FragmentColorError> {
        self.object
            .get_uniform_data(&key)
            .map_err(FragmentColorError::from)
    }

    #[uniffi::method(name = "listUniforms")]
    #[lsp_doc("docs/api/core/shader/list_uniforms.md")]
    pub fn list_uniforms_mobile(&self) -> Vec<String> {
        self.list_uniforms()
    }

    #[uniffi::method(name = "listKeys")]
    #[lsp_doc("docs/api/core/shader/list_keys.md")]
    pub fn list_keys_mobile(&self) -> Vec<String> {
        self.list_keys()
    }

    #[uniffi::method(name = "isCompute")]
    #[lsp_doc("docs/api/core/shader/is_compute.md")]
    pub fn is_compute_mobile(&self) -> bool {
        self.is_compute()
    }
}
