//! `Shader`, and `ShaderObject`, the module, uniforms and meshes a shader keeps.

pub(crate) mod error;
use crate::geometry::MeshObject;
use crate::texture::{Registration, TextureId};
use crate::{AlphaMode, Mesh, PassObject, Renderable, Vertex};
use dashmap::DashMap;
use error::Source;
pub use error::{Diagnostic, ShaderError, SourceLocation};
use lsp_doc::lsp_doc;
use naga::{
    Module,
    valid::{Capabilities, ValidationFlags, Validator},
};
use parking_lot::{Mutex, RwLock};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

mod source;
use source::defaults::*;
pub(crate) mod uniforms;
pub(crate) use uniforms::*;
pub use uniforms::{
    ArrayElement, PushEntry, StorageAccess, StorageEntry, StructField, StructShape, UniformData,
};

pub use source::{ShaderInput, ShaderPart};

pub(crate) use uniforms::input::{UniformInput, is_whole_in};

mod mesh_attach;
use mesh_attach::Preview;
mod reflection;
use reflection::{check_samplers, hash, parse_uniforms};
pub(crate) use reflection::{create_validator, parse_wgsl, validate_module};
mod vertex_inputs;
use vertex_inputs::build_wgsl_from_vertex;

mod rewrites;
pub(crate) use rewrites::immediates::lower_immediates;
pub(crate) use rewrites::struct_layouts::{LayoutLoss, list_explicit, name_entries, write_layouts};
pub(in crate::core) mod lights;
use lights::roots::LightRoots;
pub(crate) use lights::{ShaderLights, lower_cap, share_light};
mod platform;
use uniforms::bytes::*;

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
        Ok(Self::from(source::blocking::resolve(input.into())?))
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
        Ok(Self::from(source::resolve(input.into()).await?))
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
    pub fn from_vertex(vertex: &Vertex) -> Result<Self, ShaderError> {
        Ok(Self::from(Arc::new(ShaderObject::from_vertex(vertex)?)))
    }

    #[lsp_doc("docs/api/core/shader/from_mesh.md")]
    pub fn from_mesh(mesh: &Mesh) -> Result<Self, ShaderError> {
        Ok(Self::from(Arc::new(ShaderObject::from_mesh(mesh)?)))
    }

    #[lsp_doc("docs/api/core/shader/add_mesh.md")]
    pub fn add_mesh(&self, mesh: &Mesh) -> Result<(), ShaderError> {
        self.object.add_mesh(mesh.object.clone())
    }

    #[lsp_doc("docs/api/core/shader/remove_mesh.md")]
    pub fn remove_mesh(&self, mesh: &Mesh) {
        self.object.remove_mesh(&mesh.object);
    }

    #[lsp_doc("docs/api/core/shader/remove_meshes.md")]
    pub fn remove_meshes<'list, I>(&self, list: I)
    where
        I: IntoIterator<Item = &'list Mesh>,
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
    pub fn validate_mesh(&self, mesh: &Mesh) -> Result<(), ShaderError> {
        self.object.validate_mesh(&mesh.object)
    }

    #[lsp_doc("docs/api/core/shader/is_compute.md")]
    pub fn is_compute(&self) -> bool {
        self.object.is_compute()
    }

    #[lsp_doc("docs/api/core/shader/set_registry.md")]
    pub fn set_registry(base_url: &str) {
        source::registry::set_registry(base_url);
    }
}

/// FragmentColor's Shader internal implementation.
///
/// The ShaderObject is wrapped in an Arc and managed by the Shader struct.
/// This allows it to be shared between multiple passes and render pipelines.
///
/// `alpha_mode` and `double_sided` are pipeline-state back-references owned
/// by Material (see `src/scene/material/`). The renderer reads them when building
/// a `RenderPipelineKey` so different Material configurations cache to
/// distinct pipelines. They live here rather than on Material because the
/// renderer iterates `pass.shaders` (Arc<ShaderObject>) at draw time and
/// doesn't otherwise know which Material a shader belongs to.
#[derive(Debug)]
pub(crate) struct ShaderObject {
    /// Tells this shader from every other one, so a mesh keeps
    /// what each attached shader reads apart from the others.
    pub(crate) id: u64,
    pub(crate) hash: ShaderHash,
    pub(crate) module: Module,
    /// WGSL text the module was parsed from. Feeds the uniform padding
    /// pass, which rewrites declarations by source span.
    pub(crate) source: String,
    /// Module with uniform bindings padded to 16-byte multiples, built on
    /// first use by a backend that requires it. `None` when no binding
    /// needs padding or the rewrite fails.
    padded: OnceLock<Option<Module>>,
    pub(crate) storage: RwLock<UniformStorage>,
    pub(crate) total_bytes: u64,
    pub(crate) pending: DashMap<String, PendingUpdate>,
    pub(crate) meshes: RwLock<Vec<Arc<MeshObject>>>,
    pub(crate) alpha_mode: RwLock<AlphaMode>,
    pub(crate) double_sided: RwLock<bool>,
    /// The `lights` and `light_grid` roots the renderer fills, when the shader declares them.
    pub(crate) light_roots: Option<LightRoots>,
    /// The lights that reach this shader and its ambient term.
    pub(crate) lights: RwLock<ShaderLights>,
    /// Modules with the light roots lowered to a uniform block, by light count and padding.
    lowered: RwLock<HashMap<(u32, bool), Module>>,
    /// The texture uniforms filled from a source the shader keeps, such as the Material's
    /// lazy maps. Nothing drains them: each Renderer that loads one uploads a copy of its own.
    pub(crate) maps: RwLock<Vec<uniforms::texture_maps::TextureMap>>,
    /// The explicit textures bound to texture uniforms, by uniform name. Holding their
    /// registrations keeps them in the registry after their handles drop. A key is here
    /// or in `maps`, never in both.
    pub(crate) bound_textures: DashMap<String, Arc<Registration>>,
    /// Held while a texture uniform is written, made a map or bound, and while a flush
    /// applies the queue, so a setter call lands before or after each of those, never between.
    pub(crate) texture_writes: Mutex<()>,
    /// The mesh the quick view draws with this shader, and the mesh generation that
    /// last passed this shader's checks.
    preview: Mutex<Option<Preview>>,
}

impl Drop for ShaderObject {
    /// Each mesh this shader holds stops checking for what it reads.
    fn drop(&mut self) {
        for mesh in self.meshes.get_mut().iter() {
            mesh.detach(self.id);
        }
    }
}

/// What a texture write lets go of: the registration bound before, and the key's map.
/// The caller drops it after `texture_writes`, so no GPU copy is freed under that lock.
type Released = (
    Option<Arc<Registration>>,
    Option<uniforms::texture_maps::TextureMap>,
);

/// A `set` queued while the storage was busy, applied by `flush_pending`.
#[derive(Debug, Clone)]
pub(crate) enum PendingUpdate {
    /// A value already shaped to the key's declared type.
    Value(UniformData),
    /// A loose value that could not see the key's type yet.
    Input(UniformInput),
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
    /// It never waits for a draw. A texture waits while the shader binds another texture
    /// or applies its queued values, since both hold `texture_writes`, so the latest one wins.
    pub fn set(&self, key: &str, value: impl Into<UniformData>) -> Result<(), ShaderError> {
        let value: UniformData = value.into();
        if self.routes_ambient(key) {
            let input = UniformInput::try_from(value)
                .map_err(|_| ShaderError::TypeMismatch(AMBIENT_KEY.into()))?;
            return self.set_input(key, input);
        }
        let texture = matches!(&value, UniformData::Texture(_));
        let writing = texture.then(|| self.texture_writes.lock());
        let released = self.write_value(key, value)?;
        drop(writing);
        drop(released);
        Ok(())
    }

    /// Writes `value` under `key` and holds the texture it names. A texture with id 0 unsets
    /// the key, which then holds neither a map nor a texture, and another slot's map is refused.
    /// A caller writing a texture holds `texture_writes` and drops what this returns after it.
    fn write_value(&self, key: &str, value: UniformData) -> Result<Released, ShaderError> {
        let texture = match &value {
            UniformData::Texture(meta) => Some(meta.id),
            _ => None,
        };
        if let Some(id) = texture {
            self.check_bind(key, id)?;
        }
        self.write_slot(key, value)?;
        Ok(match texture {
            Some(id) => self.bind_texture(key, id),
            None => (None, None),
        })
    }

    /// Writes `value` under `key`, or queues it in `pending` when the storage is busy.
    fn write_slot(&self, key: &str, value: UniformData) -> Result<(), ShaderError> {
        if let Some(mut storage) = self.storage.try_write() {
            self.pending.remove(key);
            storage.update(key, &value)?;
        } else {
            self.pending
                .insert(key.to_string(), PendingUpdate::Value(value));
        }
        Ok(())
    }

    /// Sets a uniform from a loose value, shaped to the key's declared WGSL type.
    /// A texture waits on `texture_writes`, which a texture write and a flush hold.
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
        let writing = texture.map(|_| self.texture_writes.lock());
        if let Some(id) = texture {
            self.check_bind(key, id)?;
        }
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
        let released = texture.map(|id| self.bind_texture(key, id));
        drop(writing);
        drop(released);
        Ok(())
    }

    /// Whether `key` is the ambient term of light roots, which the shader keeps beside its lights.
    fn routes_ambient(&self, key: &str) -> bool {
        key == AMBIENT_KEY && self.light_roots.is_some()
    }

    /// Holds the texture `id` names for `key` and ends the key's map, unless `id` is
    /// that map's own. Returns the registration bound before and the map it ended.
    fn bind_texture(&self, key: &str, id: TextureId) -> Released {
        let bound = match Registration::find(id) {
            Some(registration) => self.bound_textures.insert(key.to_string(), registration),
            None => self
                .bound_textures
                .remove(key)
                .map(|(_, registration)| registration),
        };
        (bound, self.remove_map(key, id))
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
    pub fn from_vertex(vertex: &Vertex) -> Result<Self, ShaderError> {
        Self::new(&build_wgsl_from_vertex(vertex))
    }

    /// Create a shader from a mesh's first vertex, with the mesh attached.
    /// An empty mesh, and a mesh the shader refuses, fail.
    pub fn from_mesh(mesh: &Mesh) -> Result<Self, ShaderError> {
        let first = mesh.object.verts.read().first().cloned();
        let Some(vertex) = first else {
            return Err(ShaderError::InvalidKey(
                "Mesh has no vertices, and Shader::from_mesh builds a shader from the first one"
                    .into(),
            ));
        };
        let shader = Self::from_vertex(&vertex)?;
        shader.add_mesh(mesh.object.clone())?;
        Ok(shader)
    }

    /// Set the pipeline-state alpha mode for this shader. Called by
    /// `Material::alpha_mode`; the renderer reads it when building a
    /// `RenderPipelineKey`. See [`AlphaMode`].
    pub(crate) fn set_alpha_mode(&self, mode: AlphaMode) {
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
        let module = parse_wgsl(source)?;
        let info = validate_module(&module, Some(Source::wgsl(source)))?;
        rewrites::immediates::check_immediates(&module)?;
        check_samplers(&module, &info)?;

        let light_roots = lights::roots::find_roots(&module, &info)?;
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

        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let object = Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            hash,
            module,
            source: source.to_string(),
            padded: OnceLock::new(),
            storage,
            total_bytes,
            pending: DashMap::new(),
            meshes: RwLock::new(Vec::new()),
            alpha_mode: RwLock::new(AlphaMode::default()),
            // Default `double_sided=true` for shaders used directly: this
            // preserves the pre-Material renderer behavior (no back-face
            // cull) for callers who build a Shader without going through
            // Material. `Material::pbr()` flips it back to `false` so that
            // path follows the glTF 2.0 default (single-sided = back-face
            // cull on).
            double_sided: RwLock::new(true),
            maps: RwLock::new(Vec::new()),
            light_roots,
            lights: RwLock::new(ShaderLights::default()),
            lowered: RwLock::new(HashMap::new()),
            bound_textures: DashMap::new(),
            texture_writes: Mutex::new(()),
            preview: Mutex::new(None),
        };
        // Refuses vertex inputs no mesh can feed, such as two with one name.
        object.reflect_vertex_inputs()?;
        Ok(object)
    }

    /// The module a pipeline compiles on a device that takes lights through a
    /// uniform block of `uniform_lights` lights, or through storage when `None`.
    pub(crate) fn lower_module(
        &self,
        pad_uniforms: bool,
        uniform_lights: Option<u32>,
    ) -> Result<Cow<'_, Module>, ShaderError> {
        let module = self.module_for_pipeline(pad_uniforms);
        let (Some(count), Some(roots)) = (uniform_lights, self.light_roots.as_ref()) else {
            return Ok(Cow::Borrowed(module));
        };
        let key = (count, pad_uniforms);
        if let Some(lowered) = self.lowered.read().get(&key) {
            return Ok(Cow::Owned(lowered.clone()));
        }
        let lowered = lights::roots::lower_roots(module, roots, count)?;
        self.lowered.write().insert(key, lowered.clone());
        Ok(Cow::Owned(lowered))
    }

    /// Module to compile for a pipeline. Backends that reject uniform
    /// bindings not aligned to 16 bytes get the padded variant.
    pub(crate) fn module_for_pipeline(&self, pad_uniforms: bool) -> &Module {
        if !pad_uniforms {
            return &self.module;
        }
        self.padded
            .get_or_init(|| {
                let padded =
                    rewrites::uniform_padding::pad_uniform_blocks(&self.source, &self.module)?;
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

    /// Get a uniform value as Uniform struct.
    ///
    /// Blocking read for the same reason as `get_uniform_data`,
    /// so the render path and the shader's own code never fail under contention.
    pub(crate) fn get_uniform(&self, key: &str) -> Result<Uniform, ShaderError> {
        let storage = self.storage.read();
        let uniform = storage
            .get(key)
            .ok_or(ShaderError::UniformNotFound(key.into()))?;
        Ok(uniform.clone())
    }

    /// Applies the queued updates when the storage is free, or leaves them queued.
    ///
    /// A caller that needs every update applied calls it until `pending` is empty.
    /// Each update leaves the queue before it applies, so a set that lands after the flush
    /// took its key stays queued for the next one. While it applies, it holds `texture_writes`,
    /// so a texture setter or a new map waits for it to finish.
    pub(crate) fn flush_pending(&self) {
        self.apply_queue(|_| {});
    }

    /// The flush, calling `taken` with each key once its update has left the queue.
    fn apply_queue(&self, mut taken: impl FnMut(&str)) {
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
            taken(&key);
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

impl Renderable for Shader {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        vec![self.pass.clone()].into()
    }
    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        vec![self.pass.clone()].into()
    }
}

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

#[cfg(test)]
mod tests;
