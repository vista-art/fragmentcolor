pub mod error;
use crate::renderer::platform::all::CANDIDATES;
use crate::texture::{Registration, TextureId, TextureMeta};
use crate::{PassObject, Renderable};
use dashmap::DashMap;
pub use error::ShaderError;
use lsp_doc::lsp_doc;
use naga::front::wgsl::{Frontend, Options};
use naga::{
    AddressSpace, Module,
    valid::{Capabilities, ModuleInfo, ValidationFlags, Validator},
};
use parking_lot::{Mutex, RwLock};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use wgpu::DownlevelFlags;
use wgpu_naga_bridge::features_to_naga_capabilities;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod constants;
use constants::*;
pub(crate) mod uniform;
pub(crate) use uniform::*;

pub mod composer;
pub use composer::{ShaderInput, ShaderPart};

mod coerce;
pub(crate) use coerce::{UniformInput, is_whole_in};

mod attach;

pub(crate) mod registry;

pub(crate) mod embedded;

mod glsl;
mod immediates;
pub(crate) use immediates::lower_immediates;
mod layouts;
pub(crate) use layouts::{list_explicit, write_layouts};
mod input;
pub(crate) mod light_roots;
use light_roots::LightRoots;
mod lights;
#[cfg(test)]
pub(crate) use lights::lower_test_cap;
pub(crate) use lights::{ShaderLights, lower_light_cap};
mod pad_uniforms;
mod platform;
mod storage;
use storage::*;

/// The hash of a shader source.
pub type ShaderHash = [u8; 32];

/// The key of the ambient term in both light root shapes.
const AMBIENT_KEY: &str = "lights.ambient";

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/core/shader/shader.md")]
pub struct Shader {
    pub(crate) pass: Arc<PassObject>,
    pub(crate) object: Arc<ShaderObject>,
}

crate::impl_fc_kind!(Shader, "Shader");

impl Default for Shader {
    #[lsp_doc("docs/api/core/shader/default.md")]
    fn default() -> Self {
        Self::new(DEFAULT_SHADER).expect("SAFETY: DEFAULT_SHADER is built-in")
    }
}

impl From<Arc<ShaderObject>> for Shader {
    fn from(object: Arc<ShaderObject>) -> Self {
        let pass = Arc::new(PassObject::from(object.clone()));
        Self { pass, object }
    }
}

impl Shader {
    #[lsp_doc("docs/api/core/shader/new.md")]
    pub fn new(input: impl Into<ShaderInput>) -> Result<Self, ShaderError> {
        Ok(Self::from(input::blocking::resolve(input.into())?))
    }

    /// Async constructor: resolve each part of `input` by fetching URLs and
    /// registry slugs over the network, reading file paths, or using raw WGSL
    /// source verbatim. Parts are deduplicated by source hash and concatenated
    /// in order before being compiled.
    ///
    /// Use this method when one or more parts of the shader must be fetched
    /// from the network. On WASM this is the *only* way to compose shaders
    /// from remote URLs because constructors cannot be async. Python blocks
    /// the calling thread (via `pollster::block_on`); Swift and Kotlin
    /// await it as a uniffi async method.
    #[lsp_doc("docs/api/core/shader/fetch.md")]
    pub async fn fetch(input: impl Into<ShaderInput>) -> Result<Self, ShaderError> {
        Ok(Self::from(input::resolve(input.into()).await?))
    }

    #[lsp_doc("docs/api/core/shader/set.md")]
    pub fn set(&self, key: &str, value: impl Into<UniformData>) -> Result<(), ShaderError> {
        match UniformInput::try_from(value.into()) {
            Ok(input) => self.object.set_input(key, input),
            Err(data) => self.object.set(key, data),
        }
    }

    #[lsp_doc("docs/api/core/shader/get.md")]
    pub fn get<T: From<UniformData>>(&self, key: &str) -> Result<T, ShaderError> {
        Ok(self.object.get_uniform_data(key)?.into())
    }

    #[lsp_doc("docs/api/core/shader/list_uniforms.md")]
    pub fn list_uniforms(&self) -> Vec<String> {
        self.object.list_uniforms()
    }

    #[lsp_doc("docs/api/core/shader/list_keys.md")]
    pub fn list_keys(&self) -> Vec<String> {
        self.object.list_keys()
    }

    #[lsp_doc("docs/api/core/shader/from_vertex.md")]
    pub fn from_vertex(v: &crate::mesh::Vertex) -> Self {
        let src = build_wgsl_from_vertex(v);

        if let Ok(shader) = Shader::new(&src) {
            shader
        } else {
            log::error!("Failed to create shader from vertex, returning default shader");
            Shader::default()
        }
    }

    #[lsp_doc("docs/api/core/shader/from_mesh.md")]
    pub fn from_mesh(mesh: &crate::mesh::Mesh) -> Self {
        let shader_object = ShaderObject::from_mesh(mesh);
        Self::from(Arc::new(shader_object))
    }

    #[lsp_doc("docs/api/core/shader/add_mesh.md")]
    pub fn add_mesh(&self, mesh: &crate::mesh::Mesh) -> Result<(), ShaderError> {
        self.object.add_mesh(mesh.object.clone())
    }

    #[lsp_doc("docs/api/core/shader/remove_mesh.md")]
    pub fn remove_mesh(&self, mesh: &crate::mesh::Mesh) {
        self.object.remove_mesh(&mesh.object);
    }

    #[lsp_doc("docs/api/core/shader/remove_meshes.md")]
    pub fn remove_meshes<'list, I>(&self, list: I)
    where
        I: IntoIterator<Item = &'list crate::mesh::Mesh>,
    {
        for mesh in list {
            self.object.remove_mesh(&mesh.object);
        }
    }

    #[lsp_doc("docs/api/core/shader/clear_meshes.md")]
    pub fn clear_meshes(&self) {
        self.object.clear_meshes();
    }

    #[lsp_doc("docs/api/core/shader/validate_mesh.md")]
    pub fn validate_mesh(&self, mesh: &crate::mesh::Mesh) -> Result<(), ShaderError> {
        self.object.validate_mesh(&mesh.object)
    }

    #[lsp_doc("docs/api/core/shader/is_compute.md")]
    pub fn is_compute(&self) -> bool {
        self.object.is_compute()
    }

    #[lsp_doc("docs/api/core/shader/set_registry.md")]
    pub fn set_registry(base_url: &str) {
        registry::set_registry(base_url);
    }
}

/// FragmentColor's Shader internal implementation.
///
/// The ShaderObject is wrapped in an Arc and managed by the Shader struct.
/// This allows it to be shared between multiple passes and render pipelines.
///
/// `alpha_mode` and `double_sided` are pipeline-state back-references owned
/// by Material (see `src/material/`). The renderer reads them when building
/// a `RenderPipelineKey` so different Material configurations cache to
/// distinct pipelines. They live here rather than on Material because the
/// renderer iterates `pass.shaders` (Arc<ShaderObject>) at draw time and
/// doesn't otherwise know which Material a shader belongs to.
#[derive(Debug)]
pub(crate) struct ShaderObject {
    pub(crate) hash: ShaderHash,
    pub(crate) module: Module,
    /// WGSL text the module was parsed from. Feeds the uniform padding
    /// pass, which rewrites declarations by source span.
    pub(crate) source: String,
    /// Module with uniform bindings padded to 16-byte multiples, built on
    /// first use by a backend that requires it. `None` when no binding
    /// needs padding or the rewrite fails.
    padded: std::sync::OnceLock<Option<Module>>,
    pub(crate) storage: RwLock<UniformStorage>,
    pub(crate) total_bytes: u64,
    pub(crate) pending: DashMap<String, PendingUpdate>,
    pub(crate) meshes: RwLock<Vec<Arc<crate::mesh::MeshObject>>>,
    pub(crate) alpha_mode: RwLock<crate::material::AlphaMode>,
    pub(crate) double_sided: RwLock<bool>,
    /// The `lights` and `light_grid` roots the renderer fills, when the shader declares them.
    pub(crate) light_roots: Option<LightRoots>,
    /// The lights that reach this shader and its ambient term.
    pub(crate) lights: RwLock<ShaderLights>,
    /// Modules with the light roots lowered to a uniform block, by light count and padding.
    lowered: RwLock<HashMap<(u32, bool), Module>>,
    /// Texture uploads queued by lazy setters (typically the Material's
    /// `*_texture(impl Into<TextureInput>)` family). The renderer drains
    /// this list at first render and via the explicit `Renderer::load`
    /// surface; each entry becomes a real GPU texture + a
    /// `UniformData::Texture` write under the recorded key.
    pub(crate) pending_textures: RwLock<Vec<PendingTexture>>,
    /// The textures bound to texture uniforms, by uniform name. Holding their
    /// registrations keeps them in the registry after their handles drop.
    pub(crate) bound_textures: DashMap<String, Arc<Registration>>,
    /// The stamp of the latest texture setter call for each texture uniform. An upload
    /// that finishes after a newer setter call for its key is dropped, so the latest call wins.
    texture_stamps: DashMap<String, u64>,
    /// Held while a texture uniform is written or stamped. An upload checks its stamp
    /// and writes under it, so a setter call can't land between the check and the write.
    pub(crate) texture_writes: Mutex<()>,
}

/// A `set` queued while the storage was busy, applied by `flush_pending`.
#[derive(Debug, Clone)]
pub(crate) enum PendingUpdate {
    /// A value already shaped to the key's declared type.
    Value(UniformData),
    /// A loose value that could not see the key's type yet.
    Input(UniformInput),
}

/// A texture upload waiting to be realized — slot key plus the TextureInput
/// the renderer will hand to `create_texture`.
#[derive(Debug, Clone)]
pub(crate) struct PendingTexture {
    pub key: String,
    pub input: crate::TextureInput,
    /// The stamp its setter call took. A later setter for the key takes a newer one.
    pub stamp: u64,
}

impl Default for ShaderObject {
    fn default() -> Self {
        Self::new(DEFAULT_SHADER).expect("SAFETY: DEFAULT_SHADER is a known-good built-in")
    }
}

impl ShaderObject {
    /// Create a Shader object from a WGSL source string.
    /// (alias to `ShaderObject::wgsl(shader_source)`).
    ///
    /// GLSL is also supported, but you need to provide both vertex and fragment shaders:
    /// `Shader::glsl(vertex_source, fragment_source)`.
    pub fn new(source: &str) -> Result<Self, ShaderError> {
        Self::wgsl(source)
    }

    /// Sets a uniform value, now when the storage is free, or queued, last write wins.
    /// It never waits for a draw. A texture waits while the shader binds another texture.
    pub fn set(&self, key: &str, value: impl Into<UniformData>) -> Result<(), ShaderError> {
        let val: UniformData = value.into();
        if self.routes_ambient(key) {
            let input = UniformInput::try_from(val)
                .map_err(|_| ShaderError::TypeMismatch(AMBIENT_KEY.into()))?;
            return self.set_input(key, input);
        }
        let texture = matches!(&val, UniformData::Texture(meta) if meta.id.id != 0);
        let _writing = texture.then(|| self.texture_writes.lock());
        self.write_value(key, val)
    }

    /// Writes `val` under `key`, or queues it when the storage is busy, and holds the
    /// texture it names. A caller writing a texture holds `texture_writes`.
    fn write_value(&self, key: &str, val: UniformData) -> Result<(), ShaderError> {
        let texture = match &val {
            UniformData::Texture(meta) if meta.id.id != 0 => Some(meta.id),
            _ => None,
        };
        if let Some(mut storage) = self.storage.try_write() {
            self.pending.remove(key);
            storage.update(key, &val)?;
        } else {
            self.pending
                .insert(key.to_string(), PendingUpdate::Value(val));
        }
        if let Some(id) = texture {
            self.bind_texture(key, id);
        }
        Ok(())
    }

    /// Sets a uniform from a loose value, shaped to the key's declared WGSL type.
    /// A texture waits on `texture_writes` while the shader binds another texture.
    /// Nothing else waits for the storage: when it's busy the update is queued, shaped
    /// when a read lock is free, so a type error still surfaces here in the common case.
    pub(crate) fn set_input(&self, key: &str, input: UniformInput) -> Result<(), ShaderError> {
        if self.routes_ambient(key) {
            let UniformData::Vec3(color) = input.resolve(key, &UniformData::Vec3(vec![0.0; 3]))?
            else {
                return Err(ShaderError::TypeMismatch(AMBIENT_KEY.into()));
            };
            let [red, green, blue] = color[..] else {
                return Err(ShaderError::TypeMismatch(AMBIENT_KEY.into()));
            };
            self.set_ambient([red, green, blue]);
            return Ok(());
        }
        let texture = match &input {
            UniformInput::Texture(id) => Some(*id),
            _ => None,
        };
        let _writing = texture.map(|_| self.texture_writes.lock());
        if let Some(mut storage) = self.storage.try_write() {
            let value = input.resolve(key, storage.shape(key)?)?;
            self.pending.remove(key);
            storage.update(key, &value)?;
        } else {
            let update = match self.storage.try_read() {
                Some(storage) => PendingUpdate::Value(input.resolve(key, storage.shape(key)?)?),
                None => PendingUpdate::Input(input),
            };
            self.pending.insert(key.to_string(), update);
        }
        if let Some(id) = texture {
            self.bind_texture(key, id);
        }
        Ok(())
    }

    /// Whether `key` is the ambient term of light roots, which the shader keeps beside its lights.
    fn routes_ambient(&self, key: &str) -> bool {
        key == AMBIENT_KEY && self.light_roots.is_some()
    }

    /// Holds the texture `id` names for `key`, releasing the one bound before.
    fn bind_texture(&self, key: &str, id: TextureId) {
        self.stamp_texture(key);
        if let Some(registration) = Registration::find(id) {
            self.bound_textures.insert(key.to_string(), registration);
        } else {
            self.bound_textures.remove(key);
        }
    }

    // getters
    /// List all the uniforms in the shader.
    pub fn list_uniforms(&self) -> Vec<String> {
        // Blocking read to avoid empty lists under contention; writers use try_write
        let storage = self.storage.read();
        storage.list()
    }

    /// List all available keys in the shader.
    pub fn list_keys(&self) -> Vec<String> {
        if let Some(storage) = self.storage.try_read() {
            storage.keys()
        } else {
            log::warn!("Shader storage is busy, returning empty uniform keys list");
            Vec::new()
        }
    }

    /// Create a basic Shader from a vertex.
    pub fn from_vertex(v: &crate::mesh::Vertex) -> Self {
        let src = build_wgsl_from_vertex(v);

        if let Ok(shader) = Self::new(&src) {
            shader
        } else {
            log::error!("Failed to create shader from vertex, returning default shader");
            Self::default()
        }
    }

    /// Create a shader from a mesh's first vertex.
    pub fn from_mesh(mesh: &crate::mesh::Mesh) -> Self {
        let first = { mesh.object.verts.read().first().cloned() };

        let shader = if let Some(vertex) = first {
            Self::from_vertex(&vertex)
        } else {
            log::warn!("Mesh has no vertices, returning default shader");
            Self::default()
        };

        if shader.add_mesh(mesh.object.clone()).is_err() {
            log::error!("Failed to add mesh to shader created from mesh, returning empty shader");
        }

        shader
    }

    /// Add a mesh to this shader, giving its untyped attributes the types this shader declares.
    /// Every vertex and instance the mesh takes later must hold the attributes this shader reads.
    pub fn add_mesh(&self, mesh: Arc<crate::mesh::MeshObject>) -> Result<(), ShaderError> {
        let binding = self.bind_mesh(&mesh)?;
        mesh.declare(binding)?;
        self.meshes.write().push(mesh);
        Ok(())
    }

    /// Check that a mesh can attach to this shader, without attaching it or recording any type.
    pub(crate) fn validate_mesh(&self, mesh: &crate::mesh::MeshObject) -> Result<(), ShaderError> {
        self.bind_mesh(mesh).map(drop)
    }

    /// Remove a mesh from this shader.
    pub fn remove_mesh(&self, mesh: &Arc<crate::mesh::MeshObject>) {
        let mut v = self.meshes.write();
        if let Some(pos) = v.iter().position(|m| Arc::ptr_eq(m, mesh)) {
            v.remove(pos);
        }
    }

    /// Remove all meshes from this shader.
    pub fn clear_meshes(&self) {
        self.meshes.write().clear();
    }

    /// Set the pipeline-state alpha mode for this shader. Called by
    /// `Material::alpha_mode`; the renderer reads it when building a
    /// `RenderPipelineKey`. See [`crate::material::AlphaMode`].
    pub(crate) fn set_alpha_mode(&self, mode: crate::material::AlphaMode) {
        *self.alpha_mode.write() = mode;
    }

    /// Set the pipeline-state double-sided flag for this shader. Called by
    /// `Material::double_sided`; the renderer reads it to flip the
    /// `cull_mode` and key it into the pipeline cache.
    pub(crate) fn set_double_sided(&self, value: bool) {
        *self.double_sided.write() = value;
    }

    /// Reads the current value of `key` from the CPU-side bytes, in the key's declared shape.
    ///
    /// Takes a blocking read lock, which waits briefly for any in-flight `set`.
    /// The lock is released before returning, so `let v = get(); set(v)` is safe.
    /// Structs, arrays, and buffer roots read as raw bytes. See [`UniformData::decode`].
    pub fn get_uniform_data(&self, key: &str) -> Result<UniformData, ShaderError> {
        if self.routes_ambient(key) {
            return Ok(UniformData::Vec3(self.ambient().to_vec()));
        }
        let storage = self.storage.read();
        let slot = storage.shape(key)?;
        let bytes = storage
            .get_bytes(key)
            .ok_or_else(|| ShaderError::InvalidKey(key.into()))?;
        Ok(slot.decode(bytes))
    }

    /// True iff the shader declares a top-level storage binding named
    /// `root`. Used by [`crate::Renderer::read_storage`] as a precondition
    /// before issuing the GPU readback. Not part of the user-facing API —
    /// the outer `Shader` doesn't re-export this; callers reach for
    /// `Renderer::read_storage` which surfaces `StorageBindingNotFound` on
    /// the same condition.
    pub(crate) fn has_storage(&self, root: &str) -> bool {
        self.storage.read().has_storage(root)
    }

    /// The entry point a pipeline runs for `stage`: the first one declared.
    pub(crate) fn find_entry_point(&self, stage: naga::ShaderStage) -> Option<&naga::EntryPoint> {
        self.module
            .entry_points
            .iter()
            .find(|entry_point| entry_point.stage == stage)
    }

    /// Tells weather the shader is a compute shader.
    pub fn is_compute(&self) -> bool {
        self.module
            .entry_points
            .iter()
            .any(|entry_point| entry_point.stage == naga::ShaderStage::Compute)
    }

    /// Create a Shader object from a WGSL source.
    pub fn wgsl(source: &str) -> Result<Self, ShaderError> {
        let module = create_parser().parse(source)?;
        let info = create_validator().validate(&module).map_err(Box::new)?;
        immediates::check_immediates(&module)?;
        check_samplers(&module, &info)?;

        let light_roots = light_roots::find_roots(&module, &info)?;
        let filled = light_roots
            .as_ref()
            .map(LightRoots::handles)
            .unwrap_or_default();
        let uniforms = parse_uniforms(&module, &info, &filled)?;
        let storage = RwLock::new(UniformStorage::new(&uniforms)?);
        let hash = hash(source);

        let mut total_bytes = 0;
        for uniform in uniforms.values() {
            let size = uniform.data.size() as u64;
            let aligned = wgpu::util::align_to(size, 256);
            total_bytes += aligned;
        }

        let object = Self {
            hash,
            module,
            source: source.to_string(),
            padded: std::sync::OnceLock::new(),
            storage,
            total_bytes,
            pending: DashMap::new(),
            meshes: RwLock::new(Vec::new()),
            alpha_mode: RwLock::new(crate::material::AlphaMode::default()),
            // Default `double_sided=true` for shaders used directly: this
            // preserves the pre-Material renderer behavior (no back-face
            // cull) for callers who build a Shader without going through
            // Material. `Material::pbr()` flips it back to `false` so that
            // path follows the glTF 2.0 default (single-sided = back-face
            // cull on).
            double_sided: RwLock::new(true),
            pending_textures: RwLock::new(Vec::new()),
            light_roots,
            lights: RwLock::new(ShaderLights::default()),
            lowered: RwLock::new(HashMap::new()),
            bound_textures: DashMap::new(),
            texture_stamps: DashMap::new(),
            texture_writes: Mutex::new(()),
        };
        // Refuses vertex inputs no mesh can feed, such as two with one name.
        object.reflect_vertex_inputs()?;
        Ok(object)
    }

    /// The module a pipeline compiles on a device that takes lights through a
    /// uniform block of `uniform_lights` lights, or through storage when `None`.
    pub(crate) fn module_for_device(
        &self,
        pad_uniforms: bool,
        uniform_lights: Option<u32>,
    ) -> Result<std::borrow::Cow<'_, Module>, ShaderError> {
        let module = self.module_for_pipeline(pad_uniforms);
        let (Some(count), Some(roots)) = (uniform_lights, self.light_roots.as_ref()) else {
            return Ok(std::borrow::Cow::Borrowed(module));
        };
        let key = (count, pad_uniforms);
        if let Some(lowered) = self.lowered.read().get(&key) {
            return Ok(std::borrow::Cow::Owned(lowered.clone()));
        }
        let lowered = light_roots::lower_roots(module, roots, count)?;
        self.lowered.write().insert(key, lowered.clone());
        Ok(std::borrow::Cow::Owned(lowered))
    }

    /// Module to compile for a pipeline. Backends that reject uniform
    /// bindings not aligned to 16 bytes get the padded variant.
    pub(crate) fn module_for_pipeline(&self, pad_uniforms: bool) -> &Module {
        if !pad_uniforms {
            return &self.module;
        }
        self.padded
            .get_or_init(|| {
                let padded = pad_uniforms::pad_uniform_blocks(&self.source, &self.module)?;
                match naga::front::wgsl::parse_str(&padded) {
                    Ok(module) => {
                        let mut validator =
                            Validator::new(ValidationFlags::all(), Capabilities::all());
                        match validator.validate(&module) {
                            Ok(_) => Some(module),
                            Err(e) => {
                                log::warn!("padded uniform module failed validation: {e}");
                                None
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("padded uniform source failed to parse: {e}");
                        None
                    }
                }
            })
            .as_ref()
            .unwrap_or(&self.module)
    }

    /// Either set a texture uniform immediately (if the input wraps an
    /// already-uploaded `Texture` via `TextureData::CloneOf`) or queue the
    /// upload for the renderer to drain on the next `load` / `render`.
    ///
    /// Used by the Material `*_texture` setters so callers can pass either
    /// a pre-built `Texture` (eager path) or a path / bytes / URL
    /// (lazy path) through the same builder method.
    pub(crate) fn queue_or_set_texture(
        &self,
        key: impl Into<String>,
        input: crate::TextureInput,
    ) -> Result<(), ShaderError> {
        let key = key.into();
        // The latest setter call wins, so an upload queued before can't overwrite it.
        self.pending_textures
            .write()
            .retain(|entry| entry.key != key);
        if let crate::TextureData::CloneOf(ref tex) = input.data {
            let meta = TextureMeta::with_id_only(*tex.id());
            return self.set(&key, UniformData::Texture(meta));
        }
        let _writing = self.texture_writes.lock();
        let stamp = self.stamp_texture(&key);
        self.pending_textures
            .write()
            .push(PendingTexture { key, input, stamp });
        Ok(())
    }

    /// Gives `key` a new stamp, newer than every stamp before it, and returns it.
    fn stamp_texture(&self, key: &str) -> u64 {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let stamp = NEXT.fetch_add(1, Ordering::Relaxed);
        self.texture_stamps.insert(key.to_string(), stamp);
        stamp
    }

    /// Binds the texture `id` an upload for `entry` made, unless a setter call for its key
    /// came after the upload was queued. Returns whether it bound. The stamp check and the
    /// write happen under `texture_writes`, so a setter call lands before or after both.
    pub(crate) fn set_latest(
        &self,
        entry: &PendingTexture,
        id: TextureId,
    ) -> Result<bool, ShaderError> {
        let _writing = self.texture_writes.lock();
        if !self.is_latest(entry) {
            return Ok(false);
        }
        let meta = TextureMeta::with_id_only(id);
        self.write_value(&entry.key, UniformData::Texture(meta))?;
        Ok(true)
    }

    /// Whether `entry` still holds the latest setter call for its key.
    pub(crate) fn is_latest(&self, entry: &PendingTexture) -> bool {
        self.texture_stamps
            .get(&entry.key)
            .is_none_or(|stamp| *stamp == entry.stamp)
    }

    /// Take the current pending-texture queue, leaving the slot empty.
    pub(crate) fn drain_pending_textures(&self) -> Vec<PendingTexture> {
        std::mem::take(&mut *self.pending_textures.write())
    }

    /// Puts the uploads that `failed` back at the front of the queue, so the next
    /// `load` or `render` tries them again. A key set again in the meantime keeps its newer value.
    pub(crate) fn requeue_textures(&self, failed: Vec<PendingTexture>) {
        let mut queue = self.pending_textures.write();
        let kept: Vec<PendingTexture> = failed
            .into_iter()
            .filter(|entry| self.is_latest(entry))
            .filter(|entry| queue.iter().all(|queued| queued.key != entry.key))
            .collect();
        queue.splice(0..0, kept);
    }

    /// Get a uniform value as Uniform struct.
    ///
    /// Blocking read for the same reason as `get_uniform_data`. Used
    /// from the render path (`Renderer::process_*_pass`) and shader
    /// internals where the previous `try_read` would intermittently
    /// fail under stress.
    pub(crate) fn get_uniform(&self, key: &str) -> Result<Uniform, ShaderError> {
        let storage = self.storage.read();
        let uniform = storage
            .get(key)
            .ok_or(ShaderError::UniformNotFound(key.into()))?;
        Ok(uniform.clone())
    }

    /// The `@location` inputs of the vertex entry point, by location. Builtins are skipped.
    /// A struct argument gives its `@location` members, each under its own name.
    pub(crate) fn reflect_vertex_inputs(&self) -> Result<Vec<VertexInputDesc>, ShaderError> {
        let mut inputs: Vec<VertexInputDesc> = Vec::new();

        // The same entry point create_render_pipeline runs.
        if let Some(entry_point) = self.find_entry_point(naga::ShaderStage::Vertex) {
            for argument in entry_point.function.arguments.iter() {
                match argument.binding.as_ref() {
                    Some(binding) => {
                        self.push_input(&mut inputs, binding, argument.ty, &argument.name)?
                    }
                    None => self.push_members(&mut inputs, argument.ty)?,
                }
            }
        }
        // Sort by location to keep a stable order
        inputs.sort_by_key(|vertex_input| vertex_input.location);
        Ok(inputs)
    }

    /// Adds the `@location` members of the struct `ty` to `inputs`.
    fn push_members(
        &self,
        inputs: &mut Vec<VertexInputDesc>,
        ty: naga::Handle<naga::Type>,
    ) -> Result<(), ShaderError> {
        let naga::TypeInner::Struct { members, .. } = &self.module.types[ty].inner else {
            return Ok(());
        };
        for member in members {
            if let Some(binding) = member.binding.as_ref() {
                self.push_input(inputs, binding, member.ty, &member.name)?;
            }
        }
        Ok(())
    }

    /// Adds one input to `inputs` when `binding` is a `@location`, named `attr<N>` when it has no name.
    fn push_input(
        &self,
        inputs: &mut Vec<VertexInputDesc>,
        binding: &naga::Binding,
        ty: naga::Handle<naga::Type>,
        name: &Option<String>,
    ) -> Result<(), ShaderError> {
        let naga::Binding::Location { location, .. } = binding else {
            return Ok(());
        };
        let format = naga_ty_to_vertex_format(&self.module.types[ty])?;
        let name = name.clone().unwrap_or_else(|| format!("attr{location}"));
        // Each input reads the mesh attribute of its name, so two inputs can't share one.
        if let Some(first) = inputs.iter().find(|input| input.name == name) {
            return Err(ShaderError::ParseError(format!(
                "the vertex inputs at @location({}) and @location({location}) are both named `{name}`, and each input reads the mesh attribute of its name",
                first.location
            )));
        }
        inputs.push(VertexInputDesc {
            name,
            location: *location,
            format,
        });
        Ok(())
    }

    /// Applies the queued updates when the storage is free, or leaves them queued.
    ///
    /// Each update leaves the queue before it applies, so a set that lands during
    /// the flush stays queued for the next one. The flush holds `texture_writes`, so a
    /// texture setter waits for it, and an upload's stamp check and write never interleave
    /// with it. A caller that needs every update applied calls it until `pending` is empty.
    pub(crate) fn flush_pending(&self) {
        if self.pending.is_empty() {
            return;
        }
        let Some(mut storage) = self.storage.try_write() else {
            return;
        };
        let _writing = self.texture_writes.lock();
        let keys: Vec<String> = self
            .pending
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        for key in keys {
            let Some((key, update)) = self.pending.remove(&key) else {
                continue;
            };
            let value = match update {
                PendingUpdate::Value(value) => Ok(value),
                PendingUpdate::Input(input) => storage
                    .shape(&key)
                    .and_then(|slot| input.resolve(&key, slot)),
            };
            if let Err(error) = value.and_then(|value| storage.update(&key, &value)) {
                log::warn!("flush_pending: dropping update for '{key}': {error}");
            }
        }
    }
}

/// The naga capabilities of the features a device may open with, on any backend.
fn read_capabilities() -> Capabilities {
    features_to_naga_capabilities(CANDIDATES, DownlevelFlags::all())
}

/// A WGSL parser that takes only what a device may support.
fn create_parser() -> Frontend {
    let mut options = Options::new();
    options.capabilities = read_capabilities();
    Frontend::new_with_options(options)
}

/// A validator that refuses what no device FragmentColor opens can run,
/// such as `f16`, subgroups, `texture_external` or texture atomics.
pub(crate) fn create_validator() -> Validator {
    Validator::new(ValidationFlags::all(), read_capabilities())
}

fn hash(source: &str) -> ShaderHash {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let slice = hasher.finalize();

    slice.into()
}

/// Collect the set of image-class global variables that are ever consumed by an
/// `ImageSample` expression. Textures outside this set are only read via
/// `textureLoad`/image ops, and the bind-group layout can request a non-filterable
/// sample type (unlocking Rgba32Float and similar as sampled sources without
/// the `FLOAT32_FILTERABLE` feature).
///
/// If any `ImageSample` argument cannot be resolved to a direct global (e.g. it
/// comes from a function parameter), every image global is conservatively
/// reported as sampled — filterable=true is always a safe layout, filterable=false
/// is not.
fn sampled_image_globals(
    module: &Module,
) -> std::collections::HashSet<naga::Handle<naga::GlobalVariable>> {
    use naga::Expression;

    let mut sampled: std::collections::HashSet<naga::Handle<naga::GlobalVariable>> =
        Default::default();
    let mut indirect = false;

    let mut visit = |function: &naga::Function| {
        for (_, expr) in function.expressions.iter() {
            if let Expression::ImageSample { image, .. } = expr {
                match &function.expressions[*image] {
                    Expression::GlobalVariable(handle) => {
                        sampled.insert(*handle);
                    }
                    _ => indirect = true,
                }
            }
        }
    };

    for (_, f) in module.functions.iter() {
        visit(f);
    }
    for ep in &module.entry_points {
        visit(&ep.function);
    }

    if indirect {
        for (handle, var) in module.global_variables.iter() {
            if matches!(&module.types[var.ty].inner, naga::TypeInner::Image { .. }) {
                sampled.insert(handle);
            }
        }
    }

    sampled
}

/// Records that `name` holds the slot `binding` names, refusing a slot another global holds.
/// naga only refuses two globals on one slot when one entry point uses both.
fn claim_slot(
    slots: &mut HashMap<(u32, u32), String>,
    binding: &naga::ResourceBinding,
    name: &str,
) -> Result<(), ShaderError> {
    let slot = (binding.group, binding.binding);
    match slots.insert(slot, name.to_string()) {
        Some(first) => Err(ShaderError::ParseError(format!(
            "`{first}` and `{name}` both use @group({}) @binding({})",
            binding.group, binding.binding
        ))),
        None => Ok(()),
    }
}

/// The stages whose entry points use the global `handle`.
fn find_stages(
    module: &Module,
    info: &ModuleInfo,
    handle: naga::Handle<naga::GlobalVariable>,
) -> wgpu::ShaderStages {
    let mut stages = wgpu::ShaderStages::NONE;
    for (index, entry) in module.entry_points.iter().enumerate() {
        if info.get_entry_point(index)[handle].is_empty() {
            continue;
        }
        stages |= match entry.stage {
            naga::ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
            naga::ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
            naga::ShaderStage::Compute => wgpu::ShaderStages::COMPUTE,
            _ => wgpu::ShaderStages::NONE,
        };
    }
    stages
}

/// Refuses a filtering sampler that samples a depth texture. WebGPU binds a depth texture
/// only with a comparison sampler or one that doesn't filter, and a FragmentColor
/// `sampler` filters, so such a pair would draw on native backends and fail on the web.
fn check_samplers(module: &Module, info: &ModuleInfo) -> Result<(), ShaderError> {
    for index in 0..module.entry_points.len() {
        for key in info.get_entry_point(index).sampling_set.iter() {
            let image = &module.global_variables[key.image];
            let sampler = &module.global_variables[key.sampler];
            let depth = matches!(
                module.types[image.ty].inner,
                naga::TypeInner::Image {
                    class: naga::ImageClass::Depth { .. },
                    ..
                }
            );
            let filtering = matches!(
                module.types[sampler.ty].inner,
                naga::TypeInner::Sampler { comparison: false }
            );
            if depth && filtering {
                return Err(ShaderError::ParseError(format!(
                    "`{}` filters, and it samples the depth texture `{}`, which only a sampler_comparison or textureLoad reads on the web",
                    sampler.name.as_deref().unwrap_or("?"),
                    image.name.as_deref().unwrap_or("?")
                )));
            }
        }
    }
    Ok(())
}

/// Whether the global writes to its storage, as a `read_write` buffer or a writable storage texture.
fn writes_storage(module: &Module, variable: &naga::GlobalVariable) -> bool {
    match variable.space {
        AddressSpace::Storage { access } => access.contains(naga::StorageAccess::STORE),
        _ => matches!(
            module.types[variable.ty].inner,
            naga::TypeInner::Image {
                class: naga::ImageClass::Storage { access, .. },
                ..
            } if access.contains(naga::StorageAccess::STORE)
        ),
    }
}

/// Reflects the module's bindings, leaving out the `filled` globals, which the renderer fills.
fn parse_uniforms(
    module: &Module,
    info: &ModuleInfo,
    filled: &[naga::Handle<naga::GlobalVariable>],
) -> Result<HashMap<String, Uniform>, ShaderError> {
    let mut uniforms = HashMap::new();
    let mut slots: HashMap<(u32, u32), String> = HashMap::new();
    let sampled_set = sampled_image_globals(module);

    for (handle, variable) in module.global_variables.iter() {
        if filled.contains(&handle) {
            // The renderer fills a light root, which still holds its slot.
            if let (Some(name), Some(binding)) = (&variable.name, &variable.binding) {
                claim_slot(&mut slots, binding, name)?;
            }
            continue;
        }
        let stages = find_stages(module, info, handle);
        // Handle WorkGroup specially: ignore unbound; error on bound (unexpected)
        match variable.space {
            AddressSpace::WorkGroup => {
                if variable.binding.is_some() {
                    return Err(ShaderError::ParseError(
                        "Bound workgroup variables are not supported".into(),
                    ));
                } else {
                    continue;
                }
            }
            AddressSpace::Immediate => {
                // Parse immediate data (formerly WGSL push constants) as a dedicated variant;
                // no binding is expected.
                let uniform_name = variable
                    .name
                    .clone()
                    .ok_or(ShaderError::ParseError("Unnamed immediate variable".into()))?;
                let ty = &module.types[variable.ty];
                let inner = convert_type(module, ty)?;
                let span = inner.size();
                uniforms.insert(
                    uniform_name.clone(),
                    Uniform {
                        name: uniform_name,
                        group: 0,
                        binding: 0,
                        stages,
                        data: UniformData::PushConstant(vec![crate::shader::uniform::PushEntry {
                            inner: vec![inner],
                            span,
                        }]),
                    },
                );
                continue;
            }
            _ => {}
        }

        // Accept classic uniform buffers and handle-class resources (textures/samplers/storage)
        match variable.space {
            AddressSpace::Uniform | AddressSpace::Handle | AddressSpace::Storage { .. } => {}
            _ => {
                continue;
            }
        }

        let uniform_name = variable
            .name
            .clone()
            .ok_or(ShaderError::ParseError("Unnamed uniform".into()))?;

        let binding = variable
            .binding
            .as_ref()
            .ok_or(ShaderError::ParseError("Missing binding".into()))?;
        claim_slot(&mut slots, binding, &uniform_name)?;
        // No device FragmentColor opens takes writable storage in the vertex stage.
        if stages.contains(wgpu::ShaderStages::VERTEX) && writes_storage(module, variable) {
            return Err(ShaderError::ParseError(format!(
                "`{uniform_name}` is writable storage, which a @vertex entry point can't use"
            )));
        }

        let ty = &module.types[variable.ty];

        // Special handling for Storage: wrap inner shape and carry access flags
        let data = match variable.space {
            AddressSpace::Storage { access } => {
                // convert_type will yield a Struct/Array/etc. shape; wrap it
                let inner = convert_type(module, ty)?;
                let span = inner.size();
                UniformData::Storage(vec![crate::shader::uniform::StorageEntry {
                    inner: vec![inner],
                    span,
                    access: access.into(),
                }])
            }
            _ => {
                let mut d = convert_type(module, ty)?;
                if let UniformData::Texture(ref mut meta) = d {
                    meta.sampled = sampled_set.contains(&handle);
                }
                d
            }
        };

        uniforms.insert(
            uniform_name.clone(),
            Uniform {
                name: uniform_name,
                group: binding.group,
                binding: binding.binding,
                stages,
                data,
            },
        );
    }

    Ok(uniforms)
}

impl Renderable for Shader {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        vec![self.pass.clone()].into()
    }
    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        vec![self.pass.clone()].into()
    }
}

#[cfg(wasm)]
crate::impl_js_bridge!(Shader, ShaderError);

impl TryFrom<&str> for Shader {
    type Error = ShaderError;
    fn try_from(source: &str) -> Result<Self, Self::Error> {
        Self::new(source)
    }
}

impl TryFrom<String> for Shader {
    type Error = ShaderError;
    fn try_from(source: String) -> Result<Self, Self::Error> {
        Self::new(&source)
    }
}

impl TryFrom<&String> for Shader {
    type Error = ShaderError;
    fn try_from(source: &String) -> Result<Self, Self::Error> {
        Self::new(source.as_str())
    }
}

/// Descriptor for a shader vertex input parameter reflected from WGSL/Naga.
#[derive(Debug, Clone)]
pub(crate) struct VertexInputDesc {
    pub name: String,
    pub location: u32,
    pub format: wgpu::VertexFormat,
}

fn naga_ty_to_vertex_format(ty: &naga::Type) -> Result<wgpu::VertexFormat, ShaderError> {
    use naga::{ScalarKind, TypeInner, VectorSize};
    match &ty.inner {
        TypeInner::Scalar(s) => match s.kind {
            ScalarKind::Float if s.width == 4 => Ok(wgpu::VertexFormat::Float32),
            ScalarKind::Sint if s.width == 4 => Ok(wgpu::VertexFormat::Sint32),
            ScalarKind::Uint if s.width == 4 => Ok(wgpu::VertexFormat::Uint32),
            _ => Err(ShaderError::TypeMismatch(
                "Unsupported scalar width/kind for vertex input".into(),
            )),
        },
        TypeInner::Vector { size, scalar, .. } => match (size, scalar.kind, scalar.width) {
            (VectorSize::Bi, ScalarKind::Float, 4) => Ok(wgpu::VertexFormat::Float32x2),
            (VectorSize::Tri, ScalarKind::Float, 4) => Ok(wgpu::VertexFormat::Float32x3),
            (VectorSize::Quad, ScalarKind::Float, 4) => Ok(wgpu::VertexFormat::Float32x4),
            (VectorSize::Bi, ScalarKind::Sint, 4) => Ok(wgpu::VertexFormat::Sint32x2),
            (VectorSize::Tri, ScalarKind::Sint, 4) => Ok(wgpu::VertexFormat::Sint32x3),
            (VectorSize::Quad, ScalarKind::Sint, 4) => Ok(wgpu::VertexFormat::Sint32x4),
            (VectorSize::Bi, ScalarKind::Uint, 4) => Ok(wgpu::VertexFormat::Uint32x2),
            (VectorSize::Tri, ScalarKind::Uint, 4) => Ok(wgpu::VertexFormat::Uint32x3),
            (VectorSize::Quad, ScalarKind::Uint, 4) => Ok(wgpu::VertexFormat::Uint32x4),
            _ => Err(ShaderError::TypeMismatch(
                "Unsupported vector type for vertex input".into(),
            )),
        },
        // Matrices or other types are not supported as direct vertex attributes in this first pass
        _ => Err(ShaderError::TypeMismatch(
            "Unsupported vertex input type".into(),
        )),
    }
}

fn build_wgsl_from_vertex(v: &crate::mesh::Vertex) -> String {
    // Position: location 0, vec2<f32> or vec3<f32>
    let pos_ty = if v.dimensions <= 2 {
        "vec2<f32>"
    } else {
        "vec3<f32>"
    };

    // Optional color: four floats or four plain numbers, at the vertex's own location
    let mut color_decl: Option<(u32, &'static str)> = None;
    if let Some(val) = v.properties.get("color")
        && matches!(
            val,
            crate::mesh::VertexData::F32x4(_) | crate::mesh::VertexData::Number4(_)
        )
        && let Some(loc) = v.prop_locations.get("color").cloned()
    {
        color_decl = Some((loc, "vec4<f32>"));
    }

    let mut vs_inputs: Vec<String> = Vec::new();
    vs_inputs.push(format!("@location(0) pos: {}", pos_ty));
    if let Some((loc, ty)) = color_decl {
        vs_inputs.push(format!("@location({}) color: {}", loc, ty));
    }
    let vs_params = vs_inputs.join(", ");

    // VOut struct and assignments
    let mut vout_fields: Vec<String> = vec!["@builtin(position) pos: vec4<f32>".into()];
    let mut vs_assigns: Vec<String> = Vec::new();
    // position assignment
    if v.dimensions <= 2 {
        vs_assigns.push("out.pos = vec4<f32>(pos, 0.0, 1.0);".into());
    } else {
        vs_assigns.push("out.pos = vec4<f32>(pos, 1.0);".into());
    }

    let mut fs_return = "vec4<f32>(1.0,1.0,1.0,1.0)".to_string();
    if color_decl.is_some() {
        vout_fields.push("@location(0) color: vec4<f32>".into());
        vs_assigns.push("out.color = color;".into());
        fs_return = "v.color".into();
    }

    let vout_struct = format!("struct VOut {{ {} }};", vout_fields.join(", "));
    let vs_src = format!(
        "{vout}\n@vertex\nfn vs_main({params}) -> VOut {{\n  var out: VOut;\n  {assigns}\n  return out;\n}}",
        vout = vout_struct,
        params = vs_params,
        assigns = vs_assigns.join("\n  ")
    );
    let fs_src = format!(
        "@fragment\nfn fs_main(v: VOut) -> @location(0) vec4<f32> {{ return {ret}; }}",
        ret = fs_return
    );

    format!("{vs}\n{fs}", vs = vs_src, fs = fs_src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_tryfrom_str_variants() {
        let good = DEFAULT_SHADER;
        let s1 = Shader::try_from(good).unwrap();
        let s2 = Shader::try_from(good.to_string()).unwrap();
        let s3 = Shader::try_from(&good.to_string()).unwrap();

        // Touch API so they don't get optimized away
        let _ = s1.list_uniforms();
        let _ = s2.list_keys();
        let _ = s3.list_uniforms();

        let bad = "not wgsl";
        assert!(Shader::try_from(bad).is_err());
    }

    #[test]
    fn shader_new_accepts_array_of_sources() {
        let helper = "fn util() -> f32 { return 1.0; }";
        let main = r#"
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(util()); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
        "#;

        let from_array = Shader::new([helper, main]).expect("from [&str; 2]");
        let from_vec: Vec<String> = vec![helper.to_string(), main.to_string()];
        let from_vec_call = Shader::new(from_vec).expect("from Vec<String>");
        let from_slice_str: Shader = Shader::new([helper, main].as_slice()).expect("from &[&str]");

        let _ = from_array.list_uniforms();
        let _ = from_vec_call.list_keys();
        let _ = from_slice_str.list_uniforms();
    }

    // Story: Rust sets and reads matrices as flat column-major lists, as every
    // binding does, next to the nested arrays.
    #[test]
    fn set_and_get_flat_matrices() {
        let wgsl = r#"
@group(0) @binding(0) var<uniform> m4: mat4x4<f32>;
@group(0) @binding(1) var<uniform> m3: mat3x3<f32>;
@group(0) @binding(2) var<uniform> m2: mat2x2<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> {
    return m4[0] + vec4<f32>(m3[0], 0.0) + vec4<f32>(m2[0], 0.0, 0.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");

        shader.set("m4", [0.0; 16]).expect("flat mat4");
        let identity: Vec<f32> = (0..16)
            .map(|i| if i % 5 == 0 { 1.0 } else { 0.0 })
            .collect();
        shader.set("m4", identity.clone()).expect("mat4 from a Vec");
        assert_eq!(shader.get::<[f32; 16]>("m4").expect("get"), identity[..]);
        assert_eq!(shader.get::<Vec<f32>>("m4").expect("get"), identity);

        let three: [f32; 9] = std::array::from_fn(|i| i as f32);
        shader.set("m3", three).expect("flat mat3");
        assert_eq!(shader.get::<[f32; 9]>("m3").expect("get"), three);

        shader
            .set("m2", [1.0, 2.0, 3.0, 4.0])
            .expect("mat2 from four numbers");
        assert_eq!(
            shader.get::<[f32; 4]>("m2").expect("get"),
            [1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(shader.get::<Vec<u8>>("m2").expect("get").len(), 16);

        assert!(shader.set("m4", [0.0; 9]).is_err());
    }

    #[test]
    fn shader_new_accepts_owned_string() {
        let owned: String = DEFAULT_SHADER.to_string();
        let _ = Shader::new(owned).expect("from String");
    }

    #[test]
    fn shader_new_accepts_string_ref() {
        let owned: String = DEFAULT_SHADER.to_string();
        let _ = Shader::new(&owned).expect("from &String");
    }

    #[test]
    fn validate_mesh_ok_and_errors() {
        // Shader expects pos: vec3 at location 0 and offset: vec2 at location 1
        let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");

        // Compatible mesh: pos=vec3, instance offset=vec2
        let m_ok = crate::mesh::Mesh::new();
        use crate::mesh::Vertex;
        m_ok.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);
        m_ok.add_instance(Vertex::new([0.0, 0.0]).set("offset", [0.0f32, 0.0f32]));
        shader.validate_mesh(&m_ok).expect("compatible");

        // Missing attribute: no instance offset provided
        let m_missing = crate::mesh::Mesh::new();
        m_missing.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);
        let e = shader
            .validate_mesh(&m_missing)
            .expect_err("missing should err");
        match e {
            ShaderError::InvalidKey(_) => {}
            _ => panic!("unexpected error kind: {:?}", e),
        }

        // Type mismatch: instance offset present but wrong type (vec3 instead of vec2)
        let m_mismatch = crate::mesh::Mesh::new();
        m_mismatch.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);
        m_mismatch.add_instance(Vertex::new([0.0, 0.0]).set("offset", [0.0f32, 0.0f32, 0.0f32]));
        let e2 = shader
            .validate_mesh(&m_mismatch)
            .expect_err("mismatch should err");
        assert_eq!(
            e2.to_string(),
            "Mesh attribute offset: expected vec2<f32>, got vec3<f32> at instance 0"
        );
    }

    const SHADER: &str = r#"
        struct VertexOutput {
            @builtin(position) coords: vec4<f32>,
        }

        @vertex
        fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
            let x = f32(i32(in_vertex_index) - 1);
            let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
            return VertexOutput(vec4<f32>(x, y, 0.0, 1.0));
        }

        struct Circle {
            position: vec2<f32>,
            radius: f32,
            color: vec4<f32>,
        }

        @group(0) @binding(0)
        var<uniform> circle: Circle;

        @group(0) @binding(1) var<uniform> resolution: vec2<f32>;

        @fragment
        fn main(pixel: VertexOutput) -> @location(0) vec4<f32> {
            let uv = pixel.coords.xy / resolution;
            let circle_pos = circle.position.xy / resolution;
            let dist = distance(uv, circle_pos);
            let r = circle.radius / max(resolution.x, resolution.y);
            let circle_sdf = 1.0 - smoothstep(r - 0.001, r + 0.001, dist);
            return circle.color * circle_sdf;
        }
    "#;

    #[test]
    fn test_shader_should_parse_and_list_uniforms() {
        let shader = Shader::new(SHADER).unwrap();
        let mut uniforms = shader.list_uniforms();
        uniforms.sort();
        assert_eq!(uniforms, vec!["circle", "resolution"]);
    }

    #[test]
    fn test_shader_should_parse_uniforms_and_list_keys() {
        let shader = Shader::new(SHADER).unwrap();
        let mut uniforms = shader.list_keys();
        uniforms.sort();
        assert_eq!(
            uniforms,
            vec![
                "circle",
                "circle.color",
                "circle.position",
                "circle.radius",
                "resolution"
            ]
        );
    }

    #[test]
    fn test_shader_should_set_and_get_uniform() {
        let shader = Shader::new(SHADER).unwrap();
        shader.set("circle.position", [0.5, 0.5]).unwrap();
        shader.set("circle.radius", 0.25).unwrap();
        shader.set("circle.color", [1.0, 0.0, 0.0, 1.0]).unwrap();
        shader.set("resolution", [800.0, 600.0]).unwrap();

        let position: [f32; 2] = shader.get("circle.position").unwrap();
        let radius: f32 = shader.get("circle.radius").unwrap();
        let color: [f32; 4] = shader.get("circle.color").unwrap();
        let resolution: [f32; 2] = shader.get("resolution").unwrap();

        assert_eq!(position, [0.5, 0.5]);
        assert_eq!(radius, 0.25);
        assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(resolution, [800.0, 600.0]);
    }

    #[test]
    fn test_shader_should_get_uniform_raw_bytes() {
        let shader = Shader::new(SHADER).unwrap();
        shader.set("circle.position", [0.5, 0.5]).unwrap();
        shader.set("circle.radius", 0.25).unwrap();
        shader.set("circle.color", [1.0, 0.0, 0.0, 1.0]).unwrap();
        shader.set("resolution", [800.0, 600.0]).unwrap();

        let storage = shader.object.storage.read();
        let position_bytes = storage.get_bytes("circle.position").unwrap();
        let radius_bytes = storage.get_bytes("circle.radius").unwrap();
        let color_bytes = storage.get_bytes("circle.color").unwrap();
        let resolution_bytes = storage.get_bytes("resolution").unwrap();

        assert_eq!(
            position_bytes,
            [
                0x00, 0x00, 0x00, 0x3f, //
                0x00, 0x00, 0x00, 0x3f,
            ]
        );
        assert_eq!(
            radius_bytes,
            [
                0x00, 0x00, 0x80, 0x3e //
            ]
        );
        assert_eq!(
            color_bytes,
            [
                0x00, 0x00, 0x80, 0x3f, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x80, 0x3f
            ]
        );
        assert_eq!(
            resolution_bytes,
            [
                0x00, 0x00, 0x48, 0x44, //
                0x00, 0x00, 0x16, 0x44
            ]
        );
    }

    #[test]
    fn test_invalid_shader_should_return_error() {
        let result = Shader::new("invalid shader");
        assert!(result.is_err());
    }

    #[test]
    fn parses_texture_and_sampler_uniforms_meta() {
        let wgsl = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        let keys = shader.list_uniforms();
        assert!(keys.contains(&"tex".to_string()));
        assert!(keys.contains(&"samp".to_string()));

        // Verify meta
        let s = shader.object.storage.read();
        let (_, _, u_tex) = s.uniforms.get("tex").expect("tex uniform");
        match &u_tex.data {
            UniformData::Texture(meta) => {
                assert_eq!(meta.dim, crate::texture::TextureDim::D2);
                match meta.class {
                    crate::texture::TextureClass::Sampled { .. } => {}
                    _ => panic!("expected sampled image class"),
                }
            }
            _ => panic!("tex is not a texture uniform"),
        }
        let (_, _, u_samp) = s.uniforms.get("samp").expect("samp uniform");
        match &u_samp.data {
            UniformData::Sampler(info) => {
                // comparison defaults based on naga; any bool is acceptable
                let _ = info.comparison;
            }
            _ => panic!("samp is not a sampler uniform"),
        }
    }

    // Story: A simple read-only storage buffer with a single vec4 field should be parsed,
    // listed among uniforms, and preserve its naga access flags.
    #[test]
    fn storage_read_only_parses_and_lists_fields() {
        let wgsl = r#"
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read> ssbo: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(0.5,0.5,0.5,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        let mut names = shader.list_uniforms();
        names.sort();
        assert!(names.contains(&"ssbo".to_string()));

        // Verify meta
        let s = shader.object.storage.read();
        let (_, size, u) = s.uniforms.get("ssbo").expect("ssbo uniform");
        match u.data.clone() {
            UniformData::Storage(data) => {
                let entry = data.first().expect("storage data");
                let crate::shader::uniform::StorageEntry {
                    inner,
                    span,
                    access,
                } = entry;
                assert_eq!(*span, *size);
                assert!(access == &StorageAccess::Read);
                match inner.first().expect("inner shape") {
                    UniformData::Struct(s) => {
                        assert_eq!(s.size, 16); // vec4<f32>
                        // Should have one field named 'a'
                        let mut seen_a = false;
                        for f in s.fields.iter() {
                            if f.name == "a" {
                                seen_a = true;
                                assert!(matches!(
                                    f.ty.first().expect("field shape"),
                                    UniformData::Vec4(_)
                                ));
                            }
                        }
                        assert!(seen_a);
                    }
                    _ => panic!("inner shape is not a struct"),
                }
            }
            _ => panic!("ssbo is not a storage buffer uniform"),
        }
    }

    // Story: A read-write storage buffer should carry write permission in its access flags.
    #[test]
    fn storage_read_write_meta_sets_writable() {
        let wgsl = r#"
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read_write> sbuf: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(0.2,0.4,0.6,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        let s = shader.object.storage.read();
        let (_, _, u) = s.uniforms.get("sbuf").expect("sbuf uniform");
        match u.data.clone() {
            UniformData::Storage(data) => {
                let crate::shader::uniform::StorageEntry { access, .. } =
                    data.first().expect("storage data");
                assert!(access == &StorageAccess::ReadWrite);
            }
            _ => panic!("sbuf is not a storage buffer uniform"),
        }
    }

    // Story: Nested arrays and structs in a storage buffer should compute the right span.
    #[test]
    fn storage_nested_shapes_compute_span() {
        let wgsl = r#"
struct Inner { c: vec4<f32> };
struct Outer { a: vec4<f32>, arr: array<vec4<f32>, 2>, inner: Inner };
@group(0) @binding(0) var<storage, read> sto: Outer;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,0.,0.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        let s = shader.object.storage.read();
        let (_, _, u) = s.uniforms.get("sto").expect("sto uniform");
        match &u.data {
            UniformData::Storage(data) => {
                let crate::shader::uniform::StorageEntry { inner, span, .. } =
                    data.first().expect("storage data");
                // a:16 + arr:2*16 + inner.c:16 = 64
                assert_eq!(span.clone(), 64);
                match inner.first().expect("inner shape") {
                    UniformData::Struct(s) => assert_eq!(s.size, 64),
                    _ => panic!("inner is not struct"),
                }
            }
            _ => panic!("sto is not a storage buffer"),
        }
    }

    // Story: Atomic fields in a storage buffer reflect with the layout of their scalar.
    #[test]
    fn storage_atomic_fields_reflect_as_scalars() {
        let wgsl = r#"
struct A { x: atomic<i32>, y: atomic<u32> };
@group(0) @binding(0) var<storage, read_write> a: A;
@group(0) @binding(1) var<storage, read_write> counts: array<atomic<u32>, 4>;
@compute @workgroup_size(1)
fn cs_main() {
  atomicAdd(&a.x, 1);
  atomicAdd(&a.y, 1u);
  atomicAdd(&counts[0], 1u);
}
        "#;
        let shader = Shader::new(wgsl).expect("atomic storage parses");
        let s = shader.object.storage.read();

        let (_, _, a) = s.uniforms.get("a").expect("a uniform");
        let UniformData::Storage(data) = &a.data else {
            panic!("a is not a storage buffer");
        };
        let entry = data.first().expect("storage data");
        assert_eq!(entry.span, 8);
        assert_eq!(entry.access, StorageAccess::ReadWrite);
        let Some(UniformData::Struct(shape)) = entry.inner.first() else {
            panic!("a is not a struct");
        };
        let kinds: Vec<_> = shape.fields.iter().map(|field| &field.ty[0]).collect();
        assert!(matches!(
            kinds[..],
            [UniformData::Int(_), UniformData::UInt(_)]
        ));

        let (_, _, counts) = s.uniforms.get("counts").expect("counts uniform");
        let UniformData::Storage(data) = &counts.data else {
            panic!("counts is not a storage buffer");
        };
        assert_eq!(data.first().expect("storage data").span, 16);
    }

    // Story: Workgroup variables (unbound) should be ignored and not prevent parsing.
    #[test]
    fn workgroup_unbound_is_ignored() {
        let wgsl = r#"
var<workgroup> tile: array<vec4<f32>, 64>;
@compute @workgroup_size(1)
fn cs_main() { }
        "#;
        let _ = Shader::new(wgsl).expect("workgroup var ignored");
    }

    // Story: Push constants are parsed, listed, and settable just like other variables.
    #[test]
    fn push_constant_parsing_and_set() {
        let wgsl = r#"
struct PC { v: f32 };
var<immediate> pc: PC;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        let mut names = shader.list_uniforms();
        names.sort();
        assert!(names.contains(&"pc".to_string()));
        shader.set("pc.v", 0.5f32).expect("set pc.v");
        let s = shader.object.storage.read();
        let bytes = s.get_bytes("pc").expect("pc bytes");
        assert_eq!(bytes.len(), 4);
        let v: f32 = bytemuck::cast_slice(&bytes[0..4])[0];
        assert_eq!(v, 0.5f32);
    }

    // Story: Multiple push-constant roots parse and expose both roots.
    #[test]
    fn push_constant_multiple_roots_listed() {
        let wgsl = r#"
struct A { v: f32 };
struct B { c: vec4<f32> };
var<immediate> a: A;
var<immediate> b: B;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        let mut names = shader.list_uniforms();
        names.sort();
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
        // Set nested fields and verify CPU blobs updated
        shader.set("a.v", 1.0f32).expect("set a.v");
        shader.set("b.c", [0.0, 1.0, 0.0, 1.0]).expect("set b.c");
        let s = shader.object.storage.read();
        assert_eq!(s.get_bytes("a").unwrap().len(), 4);
        assert_eq!(s.get_bytes("b").unwrap().len(), 16);
    }

    // Story: Storage buffers can coexist with texture/sampler bindings.
    #[test]
    fn storage_and_texture_bindings_coexist() {
        let wgsl = r#"
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read> data: Buf;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        let mut names = shader.list_uniforms();
        names.sort();
        assert!(names.contains(&"data".to_string()));
        assert!(names.contains(&"tex".to_string()));
        assert!(names.contains(&"samp".to_string()));
    }

    // Story: Setting a storage-buffer field should update the CPU blob and be visible via get_bytes.
    #[test]
    fn storage_set_updates_cpu_blob_bytes() {
        let wgsl = r#"
struct Buf { a: vec4<f32> };
@group(0) @binding(0) var<storage, read> data: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        // Set a = (1,2,3,4)
        shader.set("data.a", [1.0, 2.0, 3.0, 4.0]).expect("set");
        let storage = shader.object.storage.read();
        let bytes = storage.get_bytes("data").expect("blob");
        assert_eq!(bytes.len(), 16);
        let expected: [u8; 16] = bytemuck::cast([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(&bytes[0..16], &expected);
    }

    // Story: Arrays in storage buffers support element indexing using naga stride.
    // - array<vec4<f32>, 4>: stride = 16 (std430); set element 2 and verify offset 2*16.
    #[test]
    fn storage_array_element_indexing() {
        let wgsl = r#"
struct Buf { arr: array<vec4<f32>, 4> };
@group(0) @binding(0) var<storage, read> data: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        // Set arr[2] = (9, 8, 7, 6)
        shader
            .set("data.arr[2]", [9.0, 8.0, 7.0, 6.0])
            .expect("set arr[2]");
        let s = shader.object.storage.read();
        let blob = s.get_bytes("data").expect("blob");
        assert_eq!(blob.len(), 4 * 16);
        // offset for index 2
        let start = 2 * 16;
        let expected: [u8; 16] = bytemuck::cast([9.0f32, 8.0, 7.0, 6.0]);
        assert_eq!(&blob[start..start + 16], &expected);
    }

    // Story: Arrays of structs in storage buffers can index into element then field.
    #[test]
    fn storage_array_of_structs_index_field() {
        let wgsl = r#"
struct Item { v: vec4<f32> };
struct Buf { items: array<Item, 3> };
@group(0) @binding(0) var<storage, read> buf: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        // Set items[1].v
        shader
            .set("buf.items[1].v", [0.1, 0.2, 0.3, 0.4])
            .expect("set items[1].v");
        let s = shader.object.storage.read();
        let blob = s.get_bytes("buf").expect("blob");
        // Expect stride >= 16; we only validate the written region matches expected bytes at index 1*stride
        // Find stride by reading through Uniform metadata
        let (_, _, u) = s.uniforms.get("buf").expect("buf uniform");
        let stride = match &u.data {
            UniformData::Storage(data) => {
                let crate::shader::uniform::StorageEntry { inner, .. } = data.first().unwrap();
                match inner.first().expect("inner shape") {
                    UniformData::Struct(s) => {
                        // fields[0] should be items: Array
                        match s.fields[0].ty.first().expect("field shape") {
                            UniformData::Array(items) => items.first().unwrap().stride,
                            _ => 0,
                        }
                    }
                    _ => 0,
                }
            }
            _ => 0,
        } as usize;
        assert!(stride >= 16);
        let start = stride; // index 1
        let expected: [u8; 16] = bytemuck::cast([0.1f32, 0.2, 0.3, 0.4]);
        assert_eq!(&blob[start..start + 16], &expected);
    }

    // Story: Top-level uniform struct with array supports element indexing in set() and get_bytes().
    #[test]
    fn uniform_array_element_indexing() {
        let wgsl = r#"
struct U { arr: array<vec4<f32>, 3> };
@group(0) @binding(0) var<uniform> u: U;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        shader
            .set("u.arr[1]", [3.0, 2.0, 1.0, 0.0])
            .expect("set u.arr[1]");
        let s = shader.object.storage.read();
        // get_bytes with element should work via slow-path computation
        let bytes = s.get_bytes("u.arr[1]").expect("bytes");
        assert_eq!(bytes.len(), 16);
        let expected: [u8; 16] = bytemuck::cast([3.0f32, 2.0, 1.0, 0.0]);
        assert_eq!(bytes, &expected);
    }

    // Story: get_bytes on storage element slice returns only that element's bytes
    #[test]
    fn storage_get_bytes_element_slice() {
        let wgsl = r#"
struct Buf { arr: array<vec4<f32>, 2> };
@group(0) @binding(0) var<storage, read> buf: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        shader
            .set("buf.arr[0]", [5.0, 6.0, 7.0, 8.0])
            .expect("set arr[0]");
        shader
            .set("buf.arr[1]", [1.0, 2.0, 3.0, 4.0])
            .expect("set arr[1]");
        let s = shader.object.storage.read();
        let e1 = s.get_bytes("buf.arr[1]").expect("bytes arr[1]");
        let expected: [u8; 16] = bytemuck::cast([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(e1, &expected);
    }

    // Story: setting a field with a wrong type returns a TypeMismatch error.
    #[test]
    fn set_with_type_mismatch_errors() {
        let wgsl = SHADER;
        let shader = Shader::new(wgsl).expect("shader");
        // circle.radius expects a float
        let err = shader
            .set("circle.radius", [1.0f32, 2.0])
            .expect_err("type mismatch");
        match err {
            ShaderError::TypeMismatch(_) => {}
            _ => panic!("unexpected error: {:?}", err),
        }
    }

    // Story: getting a non-existent key returns UniformNotFound.
    #[test]
    fn get_uniform_not_found_errors() {
        let shader = Shader::new(SHADER).expect("shader");
        let err = shader.get::<f32>("does.not.exist").expect_err("not found");
        match err {
            ShaderError::UniformNotFound(_) => {}
            _ => panic!("unexpected error: {:?}", err),
        }
    }

    // Story: array index out of bounds triggers IndexOutOfBounds for uniform arrays.
    #[test]
    fn uniform_array_index_out_of_bounds_errors() {
        let wgsl = r#"
struct U { arr: array<vec4<f32>, 2> };
@group(0) @binding(0) var<uniform> u: U;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(1.,1.,1.,1.); }
        "#;
        let shader = Shader::new(wgsl).expect("shader");
        // index 2 is out of bounds for len 2 [0,1]
        let err = shader
            .set("u.arr[2]", [1.0, 2.0, 3.0, 4.0])
            .expect_err("oob");
        match err {
            ShaderError::IndexOutOfBounds { .. } => {}
            _ => panic!("unexpected error: {:?}", err),
        }
    }

    // Story: AST-driven mapping errors when a needed attribute is missing.
    #[test]
    fn ast_mapping_missing_attribute_errors() {
        let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,0.,1.,1.); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");

        let mesh = crate::mesh::Mesh::new();
        use crate::mesh::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);

        let res = shader.add_mesh(&mesh);
        assert!(res.is_err());
        let s = format!("{}", res.unwrap_err());
        assert!(s.contains("Mesh attribute not found for shader input 'offset'"));
    }

    // Story: AST-driven mapping errors on type mismatch between shader input and mesh property format.
    #[test]
    fn ast_mapping_type_mismatch_errors() {
        let wgsl = r#"
struct VOut { @builtin(position) pos_out: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) offset: vec2<f32>) -> VOut {
  var out: VOut;
  let p = vec3<f32>(pos.xy + offset, pos.z);
  out.pos_out = vec4<f32>(p, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,1.,0.,1.); }
            "#;

        let shader = crate::Shader::new(wgsl).expect("shader");

        let mesh = crate::mesh::Mesh::new();
        use crate::mesh::{Vertex, VertexData};
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);
        // Add instance with wrong-typed "offset" (vec3 instead of vec2)
        mesh.add_instance(
            Vertex::new([0.0, 0.0]).set("offset", VertexData::F32x3([0.0, 0.0, 0.0])),
        );

        let res = shader.add_mesh(&mesh);
        assert!(res.is_err());
        let s = format!("{}", res.unwrap_err());
        assert_eq!(
            s,
            "Mesh attribute offset: expected vec2<f32>, got vec3<f32> at instance 0"
        );

        // Three plain numbers can't fill a vec2 either.
        let mesh = crate::mesh::Mesh::new();
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);
        mesh.add_instance(Vertex::new([0.0, 0.0]).set("offset", [0.0, 0.0, 0.0]));
        let s = shader.add_mesh(&mesh).expect_err("count").to_string();
        assert_eq!(
            s,
            "Mesh attribute offset: expected vec2<f32>, got 3 number(s) at instance 0"
        );
    }

    const TYPED_ID: &str = r#"
struct VOut { @builtin(position) pos: vec4<f32>, @location(0) @interpolate(flat) id: u32 };
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: u32) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 0.0, 1.0);
  out.id = id;
  return out;
}
@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(f32(v.id)); }
"#;

    const FLOAT_ID: &str = r#"
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: f32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(pos, id, 1.0);
}
@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;

    fn mesh_with_ids(ids: &[f64]) -> crate::mesh::Mesh {
        let mesh = crate::mesh::Mesh::new();
        for (index, id) in ids.iter().enumerate() {
            let x = index as f32;
            mesh.add_vertex(crate::mesh::Vertex::new([x, 0.0]).set("id", *id));
        }
        mesh
    }

    // Story: validate_mesh is a dry run. It checks the mesh, and a later
    // shader may still give its plain numbers another type.
    #[test]
    fn validate_mesh_commits_nothing() {
        let unsigned = Shader::new(TYPED_ID).expect("shader");
        let mesh = mesh_with_ids(&[1.0, 2.0, 3.0]);
        unsigned.validate_mesh(&mesh).expect("fits");
        assert!(mesh.object.types.read().vertex.is_empty());

        let float = Shader::new(FLOAT_ID).expect("shader");
        float.add_mesh(&mesh).expect("still untyped");
        assert_eq!(
            mesh.object.types.read().vertex.get("id"),
            Some(&wgpu::VertexFormat::Float32)
        );
    }

    // Story: a later vertex that lacks an attribute the shader reads fails the
    // attach and names both, where it would otherwise draw with zeros.
    #[test]
    fn a_later_vertex_without_an_attribute_fails() {
        let shader = Shader::new(FLOAT_ID).expect("shader");
        let mesh = mesh_with_ids(&[1.0, 2.0]);
        mesh.add_vertex(crate::mesh::Vertex::new([2.0, 0.0]));
        let error = shader.add_mesh(&mesh).expect_err("vertex 2 has no id");
        assert_eq!(
            error.to_string(),
            "Mesh attribute id: expected f32, got no value at vertex 2"
        );
        assert!(shader.object.meshes.read().is_empty());

        // An attribute the shader doesn't read may come and go.
        let mesh = mesh_with_ids(&[1.0, 2.0]);
        mesh.add_vertex(
            crate::mesh::Vertex::new([2.0, 0.0])
                .set("id", 3.0)
                .set("extra", 1.0),
        );
        shader.add_mesh(&mesh).expect("extra is not read");
    }

    // Story: the same holds for instances.
    #[test]
    fn a_later_instance_without_an_attribute_fails() {
        let wgsl = r#"
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) offset: vec2<f32>) -> @builtin(position) vec4<f32> {
  return vec4<f32>(pos + offset, 0.0, 1.0);
}
@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
        use crate::mesh::{Instance, Vertex};
        let shader = Shader::new(wgsl).expect("shader");
        let mesh = crate::mesh::Mesh::new();
        mesh.add_vertices([
            Vertex::new([0.0, 0.0]),
            Vertex::new([1.0, 0.0]),
            Vertex::new([0.0, 1.0]),
        ]);
        mesh.add_instance(Instance::new().set("offset", [0.0, 0.0]));
        mesh.add_instance(Instance::new().set("scale", 2.0));
        let error = shader
            .add_mesh(&mesh)
            .expect_err("instance 1 has no offset");
        assert_eq!(
            error.to_string(),
            "Mesh attribute offset: expected vec2<f32>, got no value at instance 1"
        );
    }

    // Story: an attach that fails leaves no type behind, so the next shader gets a clean try.
    #[test]
    fn failed_attach_leaves_no_types() {
        let unsigned = Shader::new(TYPED_ID).expect("shader");
        let mesh = mesh_with_ids(&[1.0, 2.0, -1.0]);
        let error = unsigned.add_mesh(&mesh).expect_err("negative u32");
        assert_eq!(
            error.to_string(),
            "Mesh attribute id: expected u32, got -1 at vertex 2"
        );
        assert!(mesh.object.types.read().vertex.is_empty());
        assert!(unsigned.object.meshes.read().is_empty());

        let float = Shader::new(FLOAT_ID).expect("shader");
        float.add_mesh(&mesh).expect("floats take -1");
        assert_eq!(float.object.meshes.read().len(), 1);
    }

    // Story: the @location members of a struct argument are vertex inputs like plain
    // @location arguments, each under its own name, so a mesh property feeds each one.
    #[test]
    fn struct_members_take_a_mesh() {
        // Arrange
        let shader = Shader::new(
            r#"
struct Extra { @builtin(vertex_index) index: u32, @location(1) offset: vec2<f32> };
@vertex fn vs_main(@location(0) pos: vec2<f32>, extra: Extra) -> @builtin(position) vec4<f32> {
  return vec4<f32>(pos + extra.offset, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#,
        )
        .expect("shader");
        let mesh = crate::mesh::Mesh::new();
        mesh.add_vertex(crate::mesh::Vertex::new([0.0, 0.0]).set("offset", [0.0, 0.0]));

        // Act
        let inputs = shader.object.reflect_vertex_inputs().expect("inputs");
        let attached = shader.add_mesh(&mesh);

        // Assert
        let names: Vec<(&str, u32)> = inputs
            .iter()
            .map(|input| (input.name.as_str(), input.location))
            .collect();
        assert_eq!(names, [("pos", 0), ("offset", 1)]);
        assert!(attached.is_ok(), "{attached:?}");
        assert_eq!(shader.object.meshes.read().len(), 1);
    }

    // Story: the shader Shader::from_vertex builds reads a color of four plain numbers.
    #[test]
    fn from_vertex_reads_an_untyped_color() {
        let vertex = crate::mesh::Vertex::new([0.0, 0.0]).set("color", [1.0, 0.0, 0.0, 1.0]);
        let shader = Shader::from_vertex(&vertex);
        let inputs = shader.object.reflect_vertex_inputs().expect("inputs");
        assert!(inputs.iter().any(|input| input.name == "color"
            && input.format == wgpu::VertexFormat::Float32x4));

        let mesh = crate::mesh::Mesh::new();
        mesh.add_vertex(vertex);
        shader.add_mesh(&mesh).expect("color attaches");
    }

    // Story: non-blocking set() enqueues while storage is write-locked; flush_pending applies updates
    #[test]
    fn set_nonblocking_when_locked_then_flush_applies() {
        // Use the existing SHADER with uniforms: circle, resolution
        let shader = Shader::new(SHADER).unwrap();

        // Lock storage to simulate contention
        let _guard = shader.object.storage.write();

        // set() should not error and should enqueue (since write lock is held)
        shader
            .set("resolution", [1024.0f32, 768.0f32])
            .expect("enqueue ok");
        shader.set("circle.radius", 0.42f32).expect("enqueue ok");

        // Drop lock and flush pending
        drop(_guard);
        shader.object.flush_pending();

        // Verify values have been applied
        let res: [f32; 2] = shader.get("resolution").expect("resolution get");
        let r: f32 = shader.get("circle.radius").expect("radius get");
        assert_eq!(res, [1024.0, 768.0]);
        assert!((r - 0.42).abs() < 1e-6);
    }

    // Story: last-wins semantics — multiple enqueued writes to the same key result in the last value
    #[test]
    fn set_last_wins_queue_semantics() {
        let shader = Shader::new(SHADER).unwrap();
        let _guard = shader.object.storage.write();

        // Enqueue several updates while locked
        for i in 0..10 {
            shader.set("circle.radius", i as f32).expect("enqueue ok");
        }
        drop(_guard);
        shader.object.flush_pending();

        // Expect the last value (9.0)
        let r: f32 = shader.get("circle.radius").expect("radius");
        assert!((r - 9.0).abs() < 1e-6);
    }

    const TYPED: &str = r#"
struct Item { v: vec4<f32> };
struct Buf { items: array<Item, 3> };
struct Arr { arr: array<vec4<f32>, 2> };
@group(0) @binding(0) var<uniform> count: u32;
@group(0) @binding(1) var<uniform> cell: vec2<i32>;
@group(0) @binding(2) var<uniform> rot: mat2x2<f32>;
@group(0) @binding(3) var<uniform> u: Arr;
@group(0) @binding(4) var<storage, read> buf: Buf;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> {
  let x = f32(count) + f32(cell.x) + rot[0].x + u.arr[1].x + buf.items[0].v.x;
  return vec4f(x, 0., 0., 1.);
}
    "#;

    // Story: plain numbers reach integer uniforms, the way JS and Python send them.
    #[test]
    fn set_writes_whole_numbers_into_integer_uniforms() {
        let shader = Shader::new(TYPED).expect("shader");
        shader
            .object
            .set_input("count", UniformInput::Number(3.0))
            .expect("u32 from a number");
        shader
            .object
            .set_input("cell", UniformInput::Numbers(vec![2.0, -1.0]))
            .expect("vec2<i32> from numbers");
        assert_eq!(shader.get::<u32>("count").expect("count"), 3);
        assert_eq!(shader.get::<[i32; 2]>("cell").expect("cell"), [2, -1]);

        // Rust takes the same path: an i32 literal fits the u32 slot.
        shader.set("count", 7).expect("u32 from an i32 literal");
        assert_eq!(shader.get::<u32>("count").expect("count"), 7);

        let err = shader.set("count", -1).expect_err("negative u32");
        assert!(err.to_string().ends_with("count: expected u32, got -1"));
    }

    // Story: a mat2x2 uniform takes four numbers from any language.
    #[test]
    fn set_builds_a_mat2_from_four_numbers() {
        let shader = Shader::new(TYPED).expect("shader");
        let values = vec![1.0, 2.0, 3.0, 4.0];
        shader
            .object
            .set_input("rot", UniformInput::Numbers(values))
            .expect("mat2x2 from four numbers");
        let rot: [[f32; 2]; 2] = shader.get("rot").expect("rot");
        assert_eq!(rot, [[1.0, 2.0], [3.0, 4.0]]);
        shader
            .set("rot", [1.0, 0.0, 0.0, 1.0])
            .expect("mat2x2 from a vec4");
    }

    // Story: a value with the wrong length is rejected and the uniform keeps its value.
    #[test]
    fn set_rejects_wrong_lengths_and_keeps_the_value() {
        let shader = Shader::new(SHADER).expect("shader");
        shader.set("resolution", [800.0, 600.0]).expect("vec2");
        let three = UniformInput::Numbers(vec![1.0, 2.0, 3.0]);
        let err = shader
            .object
            .set_input("resolution", three)
            .expect_err("three numbers for a vec2");
        assert!(matches!(err, ShaderError::TypeMismatch(_)));
        assert!(
            err.to_string()
                .ends_with("resolution: expected vec2<f32>, got 3 number(s)")
        );
        let resolution: [f32; 2] = shader.get("resolution").expect("resolution");
        assert_eq!(resolution, [800.0, 600.0]);
    }

    // Story: indexed keys check the declared type, not only the byte count.
    #[test]
    fn indexed_keys_check_the_declared_type() {
        let shader = Shader::new(TYPED).expect("shader");
        // A vec4<u32> has the 16 bytes of a vec4<f32> but the wrong type.
        let uints = UniformData::UVec4(vec![1, 2, 3, 4]);
        let err = shader
            .object
            .set("buf.items[1].v", uints.clone())
            .expect_err("vec4<u32> into vec4<f32>");
        assert!(matches!(err, ShaderError::TypeMismatch(_)));
        assert!(shader.object.set("u.arr[1]", uints).is_err());

        // Loose numbers resolve to the declared vec4<f32>.
        shader.set("buf.items[1].v", [1, 2, 3, 4]).expect("numbers");
        let v: [f32; 4] = shader.get("buf.items[1].v").expect("indexed get");
        assert_eq!(v, [1.0, 2.0, 3.0, 4.0]);
        shader
            .set("u.arr[1]", [5.0, 6.0, 7.0, 8.0])
            .expect("array element");
        let e: [f32; 4] = shader.get("u.arr[1]").expect("indexed get");
        assert_eq!(e, [5.0, 6.0, 7.0, 8.0]);

        let storage = shader.object.storage.read();
        let shape = storage.shape("buf.items[2].v").expect("shape");
        assert!(matches!(shape, UniformData::Vec4(_)));
    }

    // Story: raw bytes fill a storage root, and are rejected anywhere else.
    #[test]
    fn set_takes_bytes_only_for_buffer_roots() {
        let shader = Shader::new(TYPED).expect("shader");
        let blob: Vec<u8> = (0..48).collect();
        shader
            .object
            .set_input("buf", UniformInput::Bytes(blob.clone()))
            .expect("bytes into a storage root");
        let storage = shader.object.storage.read();
        assert_eq!(storage.get_bytes("buf").expect("blob"), &blob[..]);
        drop(storage);

        let bytes = UniformInput::Bytes(vec![0; 8]);
        assert!(shader.object.set_input("cell", bytes.clone()).is_err());
        assert!(shader.object.set_input("buf.items[0].v", bytes).is_err());
    }

    // Story: an array of structs costs one key per field, not one per element, and its elements still resolve.
    #[test]
    fn array_elements_resolve_without_keys() {
        let shader = Shader::new(
            r#"
struct Item { v: vec4<f32>, w: f32 };
struct U { items: array<Item, 4096> };
@group(0) @binding(0) var<uniform> u: U;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return u.items[0].v; }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(u.items[1].w); }
"#,
        )
        .expect("shader");
        let mut keys = shader.list_keys();
        keys.sort();
        assert_eq!(keys, ["u", "u.items"]);
        shader.set("u.items[4095].w", 2.5).expect("last element");
        assert_eq!(shader.get::<f32>("u.items[4095].w").expect("get"), 2.5);
        let err = shader.set("u.items[4096].w", 1.0).unwrap_err();
        assert!(matches!(err, ShaderError::IndexOutOfBounds { .. }), "{err}");
        // The element sits at its stride: 4095 elements of 32 bytes, then `w` after `v`.
        let bytes: Vec<u8> = shader.get::<Vec<u8>>("u").expect("bytes");
        assert_eq!(bytes.len(), 4096 * 32);
        assert_eq!(
            &bytes[4095 * 32 + 16..4095 * 32 + 20],
            &2.5f32.to_le_bytes()
        );
    }

    // Story: a shader that needs a feature no device opens with fails at Shader::new.
    #[test]
    fn features_no_device_opens_fail_at_new() {
        let frag = |body: &str| {
            format!(
                "@vertex fn vs_main() -> @builtin(position) vec4<f32> {{ return vec4<f32>(0.0); }}\n{body}"
            )
        };
        let sources = [
            frag("enable f16;\n@fragment fn fs_main() -> @location(0) vec4<f32> { let h: f16 = 1.0h; return vec4<f32>(f32(h)); }"),
            frag("@fragment fn fs_main(@builtin(primitive_index) index: u32) -> @location(0) vec4<f32> { return vec4<f32>(f32(index)); }"),
            frag("@group(0) @binding(0) var video: texture_external;\n@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(video, vec2<u32>(0u)); }"),
            "enable subgroups;\n@compute @workgroup_size(64) fn cs_main(@builtin(subgroup_size) size: u32) { }".to_string(),
        ];
        for source in sources {
            assert!(Shader::new(source.as_str()).is_err(), "{source}");
        }
        // Immediate data is one of the features a device may open with.
        let immediate = frag(
            "var<immediate> pc: vec4<f32>;\n@fragment fn fs_main() -> @location(0) vec4<f32> { return pc; }",
        );
        assert!(Shader::new(immediate.as_str()).is_ok());
    }

    // Story: an immediate root a uniform can't hold fails at Shader::new on every device,
    // since a device without native immediates binds it as a uniform.
    #[test]
    fn immediates_follow_uniform_rules() {
        let shader = |declaration: &str| {
            format!(
                "{declaration}
@vertex fn vs_main() -> @builtin(position) vec4<f32> {{ return vec4<f32>(0.0); }}
@fragment fn fs_main() -> @location(0) vec4<f32> {{ return vec4<f32>(read()); }}"
            )
        };
        let array = shader("var<immediate> pc: array<f32, 4>;\nfn read() -> f32 { return pc[1]; }");
        let field = shader(
            "struct Weights { weights: array<f32, 4> };\nvar<immediate> pc: Weights;\nfn read() -> f32 { return pc.weights[1]; }",
        );
        for source in [array, field] {
            let err = Shader::new(source.as_str()).unwrap_err().to_string();
            assert!(
                err.contains("immediate roots pc must also follow the uniform layout rules"),
                "{err}"
            );
        }
        let vector = shader("var<immediate> pc: vec4<f32>;\nfn read() -> f32 { return pc.y; }");
        assert!(Shader::new(vector.as_str()).is_ok());
    }

    // Story: two vertex inputs with one name fail at Shader::new, since both would read
    // one mesh attribute: members of two structs, or a member and a plain argument.
    #[test]
    fn vertex_inputs_with_one_name_fail() {
        let structs = r#"
struct VertexIn { @location(0) position: vec2<f32> };
struct InstanceIn { @location(1) position: vec2<f32> };
@vertex fn vs_main(v: VertexIn, i: InstanceIn) -> @builtin(position) vec4<f32> {
  return vec4<f32>(v.position + i.position, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
        let mixed = r#"
struct VertexIn { @location(0) position: vec2<f32>, @location(1) uv: vec2<f32> };
@vertex fn vs_main(v: VertexIn, @location(3) uv: vec2<f32>) -> @builtin(position) vec4<f32> {
  return vec4<f32>(v.position + v.uv + uv, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
        for (source, first, second, name) in [(structs, 0, 1, "position"), (mixed, 1, 3, "uv")] {
            let err = Shader::new(source).unwrap_err().to_string();
            let expected = format!(
                "the vertex inputs at @location({first}) and @location({second}) are both named `{name}`"
            );
            assert!(err.contains(&expected), "{err}");
        }
    }

    // Story: a filtering sampler that samples a depth texture fails at Shader::new on every
    // platform, since the web binds depth only with a comparison sampler or textureLoad.
    #[test]
    fn filtering_samplers_on_depth_fail() {
        let filtered = r#"
@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var linear: sampler;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(textureSample(depth, linear, vec2<f32>(0.5))); }
"#;
        let err = Shader::new(filtered).unwrap_err().to_string();
        assert!(
            err.contains("`linear` filters, and it samples the depth texture `depth`"),
            "{err}"
        );
        let compared = r#"
@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var shadow: sampler_comparison;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(textureSampleCompare(depth, shadow, vec2<f32>(0.5), 0.5)); }
"#;
        assert!(Shader::new(compared).is_ok());
    }

    // Story: a texture set while the upload queued for its key is in flight wins,
    // so the older upload neither binds nor goes back in the queue.
    #[test]
    fn a_setter_during_an_upload_wins() {
        let shader = Shader::new(
            r#"
@group(0) @binding(0) var image: texture_2d<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(image, vec2<i32>(0), 0); }
"#,
        )
        .expect("shader");
        let object = &shader.object;
        let lazy = crate::TextureInput::from("tests/assets/missing.png");
        object.queue_or_set_texture("image", lazy).expect("queues");
        let in_flight = object.drain_pending_textures();
        assert_eq!(in_flight.len(), 1);

        object
            .set_input("image", UniformInput::Texture(TextureId { id: 77 }))
            .expect("sets");

        assert!(!object.is_latest(&in_flight[0]), "the newer setter wins");
        object.requeue_textures(in_flight);
        assert!(
            object.pending_textures.read().is_empty(),
            "the stale upload stays out"
        );
    }

    /// A shader with one texture uniform, `image`.
    fn build_shader() -> Shader {
        Shader::new(
            r#"
@group(0) @binding(0) var image: texture_2d<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(image, vec2<i32>(0), 0); }
"#,
        )
        .expect("shader")
    }

    /// The texture id `image` holds.
    fn read_image(object: &ShaderObject) -> u64 {
        match object.get_uniform("image").map(|uniform| uniform.data) {
            Ok(UniformData::Texture(meta)) => meta.id.id,
            other => panic!("image holds no texture: {other:?}"),
        }
    }

    // Story: a finished upload binds its texture while its setter call is the latest,
    // and leaves a texture a later setter call bound.
    #[test]
    fn an_upload_binds_only_while_latest() {
        let shader = build_shader();
        let object = &shader.object;
        let lazy = || crate::TextureInput::from("tests/assets/missing.png");

        object
            .queue_or_set_texture("image", lazy())
            .expect("queues");
        let first = object.drain_pending_textures().remove(0);
        let bound = object
            .set_latest(&first, TextureId { id: 5 })
            .expect("binds");
        assert!(bound);
        assert_eq!(read_image(object), 5);

        object
            .queue_or_set_texture("image", lazy())
            .expect("queues");
        let second = object.drain_pending_textures().remove(0);
        object
            .set_input("image", UniformInput::Texture(TextureId { id: 9 }))
            .expect("sets");
        let bound = object
            .set_latest(&second, TextureId { id: 6 })
            .expect("skips");
        assert!(!bound);
        assert_eq!(
            read_image(object),
            9,
            "the older upload overwrote the setter"
        );
    }

    // Story: every texture write and stamp waits for `texture_writes`, which an upload
    // holds from its stamp check to its write, so no setter call lands in between.
    #[test]
    fn texture_writes_wait_for_an_upload() {
        use std::sync::atomic::AtomicUsize;
        let shader = build_shader();
        let object = &shader.object;
        let done = AtomicUsize::new(0);
        let count = |result: Result<(), ShaderError>| {
            done.fetch_add(1, Ordering::SeqCst);
            result
        };
        std::thread::scope(|scope| {
            let held = object.texture_writes.lock();
            let meta = TextureMeta::with_id_only(TextureId { id: 7 });
            let writers = [
                scope.spawn(|| count(object.set("image", UniformData::Texture(meta)))),
                scope.spawn(|| {
                    let id = UniformInput::Texture(TextureId { id: 8 });
                    count(object.set_input("image", id))
                }),
                scope.spawn(|| {
                    let lazy = crate::TextureInput::from("tests/assets/missing.png");
                    count(object.queue_or_set_texture("image", lazy))
                }),
            ];
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(200);
            while std::time::Instant::now() < deadline {
                let finished = done.load(Ordering::SeqCst);
                assert_eq!(finished, 0, "a texture write didn't wait for the lock");
                std::thread::yield_now();
            }
            drop(held);
            for writer in writers {
                writer.join().expect("a writer").expect("a write");
            }
        });
        assert_eq!(done.load(Ordering::SeqCst), 3);
    }

    // Story: a flush holds `texture_writes` while it applies the queue, so a texture
    // setter can't slip a newer value into the queue between its read and its removal.
    #[test]
    fn a_flush_waits_for_texture_writes() {
        use std::sync::atomic::AtomicBool;
        let shader = build_shader();
        let object = &shader.object;
        {
            let _busy = object.storage.read();
            let id = UniformInput::Texture(TextureId { id: 5 });
            object.set_input("image", id).expect("queues");
        }
        assert!(!object.pending.is_empty(), "the set was applied at once");
        let done = AtomicBool::new(false);
        std::thread::scope(|scope| {
            let held = object.texture_writes.lock();
            let flush = scope.spawn(|| {
                object.flush_pending();
                done.store(true, Ordering::SeqCst);
            });
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(200);
            while std::time::Instant::now() < deadline {
                let flushed = done.load(Ordering::SeqCst);
                assert!(!flushed, "the flush didn't wait for texture_writes");
                std::thread::yield_now();
            }
            drop(held);
            flush.join().expect("the flush");
        });
        assert!(
            object.pending.is_empty(),
            "the flush left the update queued"
        );
        assert_eq!(read_image(object), 5);
    }

    // Story: two globals on one slot fail at Shader::new, used by different entry points or not used at all.
    #[test]
    fn two_globals_on_one_slot_fail() {
        let split = r#"
@group(0) @binding(0) var<uniform> a: vec4<f32>;
@group(0) @binding(0) var<uniform> b: vec4<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return a; }
@fragment fn fs_main() -> @location(0) vec4<f32> { return b; }
"#;
        let unused = r#"
@group(1) @binding(2) var<uniform> a: vec4<f32>;
@group(1) @binding(2) var tex: texture_2d<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return a; }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
        for (source, slot) in [
            (split, "@group(0) @binding(0)"),
            (unused, "@group(1) @binding(2)"),
        ] {
            let err = Shader::new(source).unwrap_err().to_string();
            assert!(err.contains(&format!("both use {slot}")), "{err}");
        }
        let parts = Shader::new([
            "@group(0) @binding(0) var<uniform> a: vec4<f32>;",
            "@group(0) @binding(0) var<uniform> b: vec4<f32>;\n@vertex fn vs_main() -> @builtin(position) vec4<f32> { return b; }\n@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
        ]);
        assert!(parts.is_err());
        // An immediate root never takes a binding slot.
        let immediate = r#"
var<immediate> pc: vec4<f32>;
@group(0) @binding(0) var<uniform> a: vec4<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return a + pc; }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
        assert!(Shader::new(immediate).is_ok());
    }

    // Story: raw bytes of any other length than the root's leave the root as it was.
    #[test]
    fn set_refuses_bytes_that_miss_the_root_size() {
        let shader = Shader::new(TYPED).expect("shader");
        let blob: Vec<u8> = (0..48).collect();
        shader.set("buf", blob.as_slice()).expect("exact bytes");
        for length in [3, 47, 49, 96] {
            let err = shader.set("buf", vec![9u8; length]).unwrap_err();
            assert!(
                err.to_string()
                    .ends_with(&format!("buf: expected 48 bytes, got {length}")),
                "{err}"
            );
        }
        let err = shader
            .object
            .set("buf", UniformData::Bytes(vec![9; 3]))
            .unwrap_err();
        assert!(
            err.to_string().contains("expected 48 bytes, got 3"),
            "{err}"
        );
        let storage = shader.object.storage.read();
        assert_eq!(storage.get_bytes("buf").expect("blob"), &blob[..]);
    }

    // Story: get on a struct root reads the bytes as they are now, not the reflected defaults.
    #[test]
    fn get_reads_struct_roots_from_current_bytes() {
        let shader = Shader::new(SHADER).expect("shader");
        shader.set("circle.radius", 0.25).expect("radius");
        let circle = shader.object.get_uniform_data("circle").expect("circle");
        let UniformData::Bytes(bytes) = circle else {
            panic!("struct roots read as bytes, got {circle:?}");
        };
        // `radius` sits at offset 8, after `position: vec2<f32>`.
        assert_eq!(&bytes[8..12], &0.25f32.to_le_bytes());
    }

    // Story: a loose value queued while the storage is locked resolves when it flushes.
    #[test]
    fn set_queues_loose_values_while_locked() {
        let shader = Shader::new(TYPED).expect("shader");
        let guard = shader.object.storage.write();
        let count = UniformInput::Number(5.0);
        shader.object.set_input("count", count).expect("queued");
        let cell = UniformInput::Numbers(vec![1.0, 2.0, 3.0]);
        shader
            .object
            .set_input("cell", cell)
            .expect("queued, checked at flush");
        drop(guard);

        shader.object.flush_pending();
        assert!(shader.object.pending.is_empty());
        assert_eq!(shader.get::<u32>("count").expect("count"), 5);
        assert_eq!(shader.get::<[i32; 2]>("cell").expect("cell"), [0, 0]);
    }

    // Story: a shader rendered on its own is freed with its pass when dropped.
    #[test]
    fn dropped_shader_frees_after_render() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([4u32, 4u32])
                .await
                .expect("texture target");
            let shader = Shader::default();
            let object = Arc::downgrade(&shader.object);
            let pass = Arc::downgrade(&shader.pass);

            renderer.render(&shader, &target).expect("render");
            drop(shader);

            assert!(object.upgrade().is_none(), "the shader leaked");
            assert!(pass.upgrade().is_none(), "the shader's pass leaked");
        });
    }
}
