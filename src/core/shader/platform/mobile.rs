//! Mobile (Swift / Kotlin) uniffi bindings for `Shader`.
//!
//! Uniffi cannot express `impl Into<ShaderInput>` over the FFI boundary, so we
//! expose two named constructors: `new(source)` for the single-string form and
//! `compose(parts)` for the array form. Each input runs through the same
//! classifier (raw source / slug / URL / path) on the Rust side. Swift / Kotlin
//! extension shims provide a single overloaded `Shader(_:)` that picks the
//! right constructor based on argument type.
//!
//! The mesh-aware constructors (`fromMesh`, `fromVertex`) and instance methods
//! (`addMesh`, `removeMesh`, `removeMeshes`, `clearMeshes`, `validateMesh`)
//! accept `Arc<Mesh>` / `Arc<Vertex>` directly — uniffi natively marshals
//! `Arc<T>` for any `uniffi::Object`.

use std::sync::Arc;

use lsp_doc::lsp_doc;

use crate::Shader;
use crate::UniformData;
use crate::core::renderer::platform::mobile::FragmentColorError;
use crate::core::shader::UniformInput;
use crate::mesh::{Mesh, Vertex};
use crate::texture::TextureId;

/// A `Shader.set` value as Swift and Kotlin hand it over.
/// The uniform's declared WGSL type decides how Rust reads it.
#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum UniformValue {
    /// Writes 1 or 0 into a numeric uniform.
    Bool(bool),
    /// One `f32`, `i32` or `u32`.
    Number(f64),
    /// A vector, a matrix, or a one-element scalar.
    Numbers(Vec<f64>),
    /// A texture created by a `Renderer`.
    Texture(TextureId),
    /// Raw bytes for a storage-buffer or push-constant root.
    Bytes(Vec<u8>),
}

impl From<UniformValue> for UniformInput {
    fn from(value: UniformValue) -> Self {
        match value {
            UniformValue::Bool(flag) => Self::Bool(flag),
            UniformValue::Number(number) => Self::Number(number),
            UniformValue::Numbers(numbers) => Self::Numbers(numbers),
            UniformValue::Texture(id) => Self::Texture(id),
            UniformValue::Bytes(bytes) => Self::Bytes(bytes),
        }
    }
}

#[uniffi::export]
impl Shader {
    /// Foreign bindings see this as `Shader.new(source)`. On Swift, uniffi
    /// generates a `convenience init` when the constructor name is `new`,
    /// so callers write `let shader = try Shader(source: "...")`. Kotlin
    /// keeps the same factory signature.
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/core/shader/hidden/new_mobile.md")]
    pub fn new_mobile(source: String) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(Shader::new(source)?))
    }

    /// Compose a shader from an array of parts (raw sources, slugs, URLs, paths).
    /// Each entry is classified independently; results are deduplicated and
    /// concatenated in order.
    #[uniffi::constructor(name = "compose")]
    #[lsp_doc("docs/api/core/shader/hidden/compose_mobile.md")]
    pub fn compose_mobile(parts: Vec<String>) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(Shader::new(parts)?))
    }

    #[uniffi::constructor(name = "default")]
    #[lsp_doc("docs/api/core/shader/default.md")]
    pub fn default_mobile() -> Arc<Self> {
        Arc::new(Shader::default())
    }

    /// Override the slug registry base URL (process-wide).
    /// Uniffi has no static-method form on `uniffi::Object`, so this is
    /// expressed as a constructor that performs the side effect and returns
    /// a default `Shader`. It carries its own name so the Swift and Kotlin
    /// extensions can define `Shader.setRegistry(baseUrl)` without a return
    /// value, matching the JS / Python static-method spelling.
    #[uniffi::constructor(name = "useRegistry")]
    #[lsp_doc("docs/api/core/shader/hidden/use_registry_mobile.md")]
    pub fn use_registry_mobile(base_url: String) -> Arc<Self> {
        Shader::set_registry(&base_url);
        Arc::new(Shader::default())
    }

    /// Async fetch constructor: resolve each part of `input` (URL, slug, file
    /// path, or raw source) over the network, then compile the merged WGSL.
    ///
    /// Expressed as an async method rather than a constructor. Swift / Kotlin
    /// extension shims provide `Shader.fetch(input:)` as a static async factory
    /// so callers never need to hold a dummy instance.
    ///
    /// Swift callers: `let shader = try await Shader.fetch("https://...")`
    /// Kotlin callers: `val shader = Shader.fetch("https://...")`
    #[uniffi::method(name = "fetch")]
    #[lsp_doc("docs/api/core/shader/hidden/fetch_mobile.md")]
    pub async fn fetch_mobile(
        self: Arc<Self>,
        input: String,
    ) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(Shader::fetch(input).await?))
    }

    /// Async fetch constructor (multi-part): resolve each element of `parts`
    /// independently then compile the merged WGSL. Mirrors `Shader.compose`
    /// but fetches remote parts asynchronously.
    #[uniffi::method(name = "fetchCompose")]
    #[lsp_doc("docs/api/core/shader/hidden/fetch_compose_mobile.md")]
    pub async fn fetch_compose_mobile(
        self: Arc<Self>,
        parts: Vec<String>,
    ) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(Shader::fetch(parts).await?))
    }

    #[uniffi::method(name = "set")]
    #[lsp_doc("docs/api/core/shader/set.md")]
    pub fn set_mobile(&self, key: String, value: UniformValue) -> Result<(), FragmentColorError> {
        Ok(self.object.set_input(&key, value.into())?)
    }

    #[uniffi::method(name = "get")]
    #[lsp_doc("docs/api/core/shader/get.md")]
    pub fn get_mobile(&self, key: String) -> Result<UniformData, FragmentColorError> {
        Ok(self.object.get_uniform_data(&key)?)
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

    // ------------------------------------------------------------------
    // Mesh-aware constructors
    // ------------------------------------------------------------------

    /// Build a basic WGSL shader from the first vertex in a `Mesh`.
    /// Automatically attaches the mesh so callers don't need a separate
    /// `addMesh` call. Mobile shim because uniffi cannot marshal `&Mesh`
    /// (shared reference) across the FFI boundary — it requires `Arc<T>`.
    #[uniffi::constructor(name = "fromMesh")]
    #[lsp_doc("docs/api/core/shader/hidden/from_mesh_mobile.md")]
    pub fn from_mesh_mobile(mesh: Arc<Mesh>) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(Shader::from_mesh(&mesh)?))
    }

    /// Build a basic WGSL shader from a single `Vertex` layout.
    /// Mobile shim because uniffi cannot marshal `&Vertex` across the FFI
    /// boundary — it requires `Arc<T>`.
    #[uniffi::constructor(name = "fromVertex")]
    #[lsp_doc("docs/api/core/shader/hidden/from_vertex_mobile.md")]
    pub fn from_vertex_mobile(vertex: Arc<Vertex>) -> Result<Arc<Self>, FragmentColorError> {
        Ok(Arc::new(Shader::from_vertex(&vertex)?))
    }

    // ------------------------------------------------------------------
    // Mesh-aware instance methods
    // ------------------------------------------------------------------

    /// Attach a `Mesh` to this shader, or return an error and leave it detached.
    /// Meshes with the same layout share a pipeline, and each other layout gets its own.
    #[uniffi::method(name = "addMesh")]
    #[lsp_doc("docs/api/core/shader/add_mesh.md")]
    pub fn add_mesh_mobile(self: Arc<Self>, mesh: Arc<Mesh>) -> Result<(), FragmentColorError> {
        Ok(self.add_mesh(&mesh)?)
    }

    /// Detach a specific `Mesh` from this shader.
    #[uniffi::method(name = "removeMesh")]
    #[lsp_doc("docs/api/core/shader/remove_mesh.md")]
    pub fn remove_mesh_mobile(self: Arc<Self>, mesh: Arc<Mesh>) {
        self.remove_mesh(&mesh);
    }

    /// Detach a list of meshes from this shader.
    #[uniffi::method(name = "removeMeshes")]
    #[lsp_doc("docs/api/core/shader/remove_meshes.md")]
    pub fn remove_meshes_mobile(self: Arc<Self>, meshes: Vec<Arc<Mesh>>) {
        for m in meshes {
            self.remove_mesh(&m);
        }
    }

    /// Detach all meshes from this shader.
    #[uniffi::method(name = "clearMeshes")]
    #[lsp_doc("docs/api/core/shader/clear_meshes.md")]
    pub fn clear_meshes_mobile(self: Arc<Self>) {
        self.clear_meshes();
    }

    /// Check whether a `Mesh`'s vertex/instance layout is compatible with
    /// this shader's `@location` inputs — without attaching it.
    #[uniffi::method(name = "validateMesh")]
    #[lsp_doc("docs/api/core/shader/validate_mesh.md")]
    pub fn validate_mesh_mobile(
        self: Arc<Self>,
        mesh: Arc<Mesh>,
    ) -> Result<(), FragmentColorError> {
        Ok(self.validate_mesh(&mesh)?)
    }
}
