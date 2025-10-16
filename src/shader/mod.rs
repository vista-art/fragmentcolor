pub mod error;
use crate::{PassObject, Renderable};
use dashmap::DashMap;
pub use error::ShaderError;
use lsp_doc::lsp_doc;
use naga::{
    AddressSpace, Module,
    valid::{Capabilities, ValidationFlags, Validator},
};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

pub mod constants;
pub use constants::*;
pub(crate) mod uniform;
pub(crate) use uniform::*;

mod glsl;
mod input;
mod platform;
mod storage;
use storage::*;

/// The hash of a shader source.
pub type ShaderHash = [u8; 32];

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/core/shader/shader.md")]
pub struct Shader {
    pub(crate) pass: Arc<PassObject>,
    pub(crate) object: Arc<ShaderObject>,
}

crate::impl_fc_kind!(Shader, "Shader");

impl Default for Shader {
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
    pub fn new(source: &str) -> Result<Self, ShaderError> {
        Ok(Self::from(input::load_shader(source)?))
    }

    #[lsp_doc("docs/api/core/shader/set.md")]
    pub fn set(&self, key: &str, value: impl Into<UniformData>) -> Result<(), ShaderError> {
        self.object.set(key, value)
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
        self.object.validate_mesh(mesh.object.clone())
    }

    #[lsp_doc("docs/api/core/shader/is_compute.md")]
    pub fn is_compute(&self) -> bool {
        self.object.is_compute()
    }
}

/// FragmentColor's Shader internal implementation.
///
/// The ShaderObject is wrapped in an Arc and managed by the Shader struct.
/// This allows it to be shared between multiple passes and render pipelines.
#[derive(Debug)]
pub(crate) struct ShaderObject {
    pub(crate) hash: ShaderHash,
    pub(crate) module: Module,
    pub(crate) storage: RwLock<UniformStorage>,
    pub(crate) total_bytes: u64,
    pub(crate) pending: DashMap<String, UniformData>,
    pub(crate) meshes: RwLock<Vec<Arc<crate::mesh::MeshObject>>>,
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
    ///
    /// Shadertoy-flavored GLSL is supported in the toy() constructor.
    /// In this case, a default vertex shader is provided automatically:
    /// `Shader::toy(shadertoy_source)`.
    pub fn new(source: &str) -> Result<Self, ShaderError> {
        Self::wgsl(source)
    }

    /// Set a uniform value.
    /// Non-blocking: applies immediately when storage is available, otherwise enqueues a last-wins update.
    pub fn set(&self, key: &str, value: impl Into<UniformData>) -> Result<(), ShaderError> {
        let val: UniformData = value.into();
        if let Some(mut storage) = self.storage.try_write() {
            storage.update(key, &val)
        } else {
            self.pending.insert(key.to_string(), val);
            Ok(())
        }
    }

    // getters
    /// List all the uniforms in the shader.
    pub fn list_uniforms(&self) -> Vec<String> {
        if let Some(storage) = self.storage.try_read() {
            storage.list()
        } else {
            log::warn!("Shader storage is busy, returning empty uniforms list");
            Vec::new()
        }
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

    /// Add a mesh to this shader.
    pub fn add_mesh(&self, mesh: Arc<crate::mesh::MeshObject>) -> Result<(), ShaderError> {
        self.validate_mesh(mesh.clone())?;
        self.meshes.write().push(mesh);
        Ok(())
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

    /// Get a uniform value as UniformData enum.
    pub fn get_uniform_data(&self, key: &str) -> Result<UniformData, ShaderError> {
        let storage = self
            .storage
            .try_read()
            .ok_or_else(|| ShaderError::Busy("storage read".into()))?;
        let uniform = storage
            .get(key)
            .ok_or(ShaderError::UniformNotFound(key.into()))?;
        Ok(uniform.data.clone())
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
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        let module = naga::front::wgsl::parse_str(source)?;
        validator.validate(&module).map_err(Box::new)?;

        let uniforms = parse_uniforms(&module)?;
        let storage = RwLock::new(UniformStorage::new(&uniforms));
        let hash = hash(source);

        let mut total_bytes = 0;
        for uniform in uniforms.values() {
            let size = uniform.data.size() as u64;
            let aligned = wgpu::util::align_to(size, 256);
            total_bytes += aligned;
        }

        Ok(Self {
            hash,
            module,
            storage,
            total_bytes,
            pending: DashMap::new(),
            meshes: RwLock::new(Vec::new()),
        })
    }

    /// Get a uniform value as Uniform struct.
    pub(crate) fn get_uniform(&self, key: &str) -> Result<Uniform, ShaderError> {
        let storage = self
            .storage
            .try_read()
            .ok_or_else(|| ShaderError::Busy("storage read".into()))?;
        let uniform = storage
            .get(key)
            .ok_or(ShaderError::UniformNotFound(key.into()))?;
        Ok(uniform.clone())
    }

    /// Validate that a Mesh is compatible with this Shader's vertex inputs.
    pub(crate) fn validate_mesh(
        &self,
        mesh: Arc<crate::mesh::MeshObject>,
    ) -> Result<(), ShaderError> {
        let inputs = self.reflect_vertex_inputs()?;
        if inputs.is_empty() {
            return Err(ShaderError::InvalidKey(
                "
                Invalid Mesh: Shader has no vertex inputs and renders fullscreen.\n

                To create a compatible Shader, use Shader::from_vertex() or Shader::from_mesh(),
                or provide a vertex shader with at least a 'position' input.
                "
                .into(),
            ));
        }

        use std::collections::HashMap;
        // Build maps from the first vertex
        let (pos_fmt_opt, v_loc_map, v_name_map) = {
            let verts = mesh.verts.read();
            if let Some(v) = verts.first() {
                let pos_fmt = if v.dimensions <= 2 {
                    wgpu::VertexFormat::Float32x2
                } else {
                    wgpu::VertexFormat::Float32x3
                };
                // location -> (name, fmt) for vertex properties; position is handled specially at loc 0
                let mut loc: HashMap<u32, (String, wgpu::VertexFormat)> = HashMap::new();
                for (name, loc_idx) in v.prop_locations.iter() {
                    if let Some(val) = v.properties.get(name) {
                        loc.insert(*loc_idx, (name.clone(), val.format()));
                    }
                }
                // name -> fmt, include position under name "position"
                let mut by_name: HashMap<String, wgpu::VertexFormat> = HashMap::new();
                by_name.insert("position".to_string(), pos_fmt);
                for (name, val) in v.properties.iter() {
                    by_name.insert(name.clone(), val.format());
                }
                (Some(pos_fmt), loc, by_name)
            } else {
                // No vertices present: cannot satisfy vertex inputs
                (None, HashMap::new(), HashMap::new())
            }
        };

        // Build maps from the first instance (if present)
        let (i_loc_map, i_name_map) = {
            let insts = mesh.insts.read();
            if let Some(i) = insts.first() {
                let mut loc: HashMap<u32, (String, wgpu::VertexFormat)> = HashMap::new();
                let mut by_name: HashMap<String, wgpu::VertexFormat> = HashMap::new();
                for (name, loc_idx) in i.prop_locations.iter() {
                    if let Some(val) = i.properties.get(name) {
                        loc.insert(*loc_idx, (name.clone(), val.format()));
                        by_name.insert(name.clone(), val.format());
                    }
                }
                (loc, by_name)
            } else {
                (HashMap::new(), HashMap::new())
            }
        };

        // position location is assumed to be 0
        let pos_loc: u32 = 0;

        // Validate each shader input
        for inp in inputs.iter() {
            let mut matched = false;

            // 1) Try instance by explicit location
            if let Some((_, f)) = i_loc_map.get(&inp.location) {
                if *f == inp.format {
                    matched = true;
                } else {
                    return Err(ShaderError::TypeMismatch(format!(
                        "Type mismatch for shader input '{}' @location({}): shader expects {:?}, mesh has {:?}",
                        inp.name, inp.location, inp.format, *f
                    )));
                }
            }

            // 2) Try vertex by explicit location (position or other property)
            if !matched {
                if inp.location == pos_loc {
                    if let Some(pos_fmt) = pos_fmt_opt {
                        if pos_fmt == inp.format {
                            matched = true;
                        } else {
                            return Err(ShaderError::TypeMismatch(format!(
                                "Type mismatch for vertex 'position' @location({}): shader expects {:?}, mesh has {:?}",
                                inp.location, inp.format, pos_fmt
                            )));
                        }
                    } else {
                        return Err(ShaderError::InvalidKey(
                            "Mesh has no vertices to provide 'position'".into(),
                        ));
                    }
                } else if let Some((_, f)) = v_loc_map.get(&inp.location) {
                    if *f == inp.format {
                        matched = true;
                    } else {
                        return Err(ShaderError::TypeMismatch(format!(
                            "Type mismatch for shader input '{}' @location({}): shader expects {:?}, mesh has {:?}",
                            inp.name, inp.location, inp.format, *f
                        )));
                    }
                }
            }

            // 3) Fallback by name: instance first, then vertex
            if !matched {
                if let Some(fmt) = i_name_map.get(&inp.name) {
                    if *fmt == inp.format {
                        matched = true;
                    } else {
                        return Err(ShaderError::TypeMismatch(format!(
                            "Type mismatch for shader input '{}' by name: shader expects {:?}, mesh has {:?}",
                            inp.name, inp.format, fmt
                        )));
                    }
                } else if let Some(fmt) = v_name_map.get(&inp.name) {
                    if *fmt == inp.format {
                        matched = true;
                    } else {
                        return Err(ShaderError::TypeMismatch(format!(
                            "Type mismatch for shader input '{}' by name: shader expects {:?}, mesh has {:?}",
                            inp.name, inp.format, fmt
                        )));
                    }
                }
            }

            if !matched {
                return Err(ShaderError::InvalidKey(format!(
                    "Mesh attribute not found for shader input '{}' @location({})",
                    inp.name, inp.location
                )));
            }
        }

        Ok(())
    }

    /// Reflect the vertex entry-point inputs as (name, location, format).
    /// Returns only parameters with @location decorations; builtins are ignored.
    pub(crate) fn reflect_vertex_inputs(&self) -> Result<Vec<VertexInputDesc>, ShaderError> {
        // Find the vertex entry point (assume first if multiple; consistent with create_render_pipeline).
        let mut inputs: Vec<VertexInputDesc> = Vec::new();

        // Iterate entry points and collect from the vertex stage only once (first hit wins)
        for entry_point in self.module.entry_points.iter() {
            if entry_point.stage != naga::ShaderStage::Vertex {
                continue;
            }

            for argument in entry_point.function.arguments.iter() {
                // Only consider @location bindings
                let Some(binding) = argument.binding.as_ref() else {
                    continue;
                };
                let naga::Binding::Location { location, .. } = binding else {
                    continue;
                };
                let ty = &self.module.types[argument.ty];
                let format = naga_ty_to_vertex_format(ty)?;
                let name = argument
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("attr{}", location));
                inputs.push(VertexInputDesc {
                    name,
                    location: *location,
                    format,
                });
            }
            break;
        }
        // Sort by location to keep a stable order
        inputs.sort_by_key(|vertex_input| vertex_input.location);
        Ok(inputs)
    }

    /// Apply any queued updates if we can acquire the write lock;
    /// otherwise leave them queued for later.
    pub(crate) fn flush_pending(&self) {
        if self.pending.is_empty() {
            return;
        }
        if let Some(mut storage) = self.storage.try_write() {
            let updates: Vec<(String, UniformData)> = self
                .pending
                .iter()
                .map(|e| (e.key().clone(), e.value().clone()))
                .collect();
            for (k, v) in updates.into_iter() {
                if let Err(e) = storage.update(&k, &v) {
                    log::warn!("flush_pending: dropping update for '{}': {}", k, e);
                }
                self.pending.remove(&k);
            }
        }
    }
}

fn hash(source: &str) -> ShaderHash {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let slice = hasher.finalize();

    slice.into()
}

fn parse_uniforms(module: &Module) -> Result<HashMap<String, Uniform>, ShaderError> {
    let mut uniforms = HashMap::new();

    for (_, variable) in module.global_variables.iter() {
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
            AddressSpace::PushConstant => {
                // Parse push constants as a dedicated variant; no binding expected
                let uniform_name = variable
                    .name
                    .clone()
                    .ok_or(ShaderError::ParseError("Unnamed push constant".into()))?;
                let ty = &module.types[variable.ty];
                let inner = convert_type(module, ty)?;
                let span = inner.size();
                uniforms.insert(
                    uniform_name.clone(),
                    Uniform {
                        name: uniform_name,
                        group: 0,
                        binding: 0,
                        data: UniformData::PushConstant(vec![(inner, span)]),
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

        let ty = &module.types[variable.ty];

        // Special handling for Storage: wrap inner shape and carry access flags
        let data = match variable.space {
            AddressSpace::Storage { access } => {
                // convert_type will yield a Struct/Array/etc. shape; wrap it
                let inner = convert_type(module, ty)?;
                let span = inner.size();
                UniformData::Storage(vec![(inner, span, access.into())])
            }
            _ => convert_type(module, ty)?,
        };

        uniforms.insert(
            uniform_name.clone(),
            Uniform {
                name: uniform_name,
                group: binding.group,
                binding: binding.binding,
                data,
            },
        );
    }

    Ok(uniforms)
}

impl Renderable for Shader {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        // Ensure cached order for this pass is up-to-date
        crate::pass::PassObject::ensure_flat_current(&self.pass);
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

    // Optional color: if present and F32x4, capture its location
    let mut color_decl: Option<(u32, &'static str)> = None;
    if let Some(val) = v.properties.get("color")
        && matches!(val, crate::mesh::VertexValue::F32x4(_))
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
        match e2 {
            ShaderError::TypeMismatch(_) => {}
            _ => panic!("unexpected error kind: {:?}", e2),
        }
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
                assert_eq!(meta.dim, naga::ImageDimension::D2);
                match meta.class {
                    naga::ImageClass::Sampled { .. } => {}
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
                let (inner, span, access) = data.first().expect("storage data");
                assert_eq!(*span, *size);
                assert!(access == &StorageAccess::Read);
                match inner {
                    UniformData::Struct((fields, s)) => {
                        assert_eq!(s.clone(), 16); // vec4<f32>
                        // Should have one field named 'a'
                        let mut seen_a = false;
                        for (_ofs, name, f) in fields.iter() {
                            if name == "a" {
                                seen_a = true;
                                assert!(matches!(f, UniformData::Vec4(_)));
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
                let (_, _, access) = data.first().expect("storage data");
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
                let (inner, span, _) = data.first().expect("storage data");
                // a:16 + arr:2*16 + inner.c:16 = 64
                assert_eq!(span.clone(), 64);
                match inner {
                    UniformData::Struct((_fields, s)) => assert_eq!(s.clone(), 64),
                    _ => panic!("inner is not struct"),
                }
            }
            _ => panic!("sto is not a storage buffer"),
        }
    }

    // Story: Atomics in storage buffers are not yet supported; parsing should return an error today.
    #[test]
    fn storage_atomic_fields_currently_unsupported() {
        let wgsl = r#"
struct A { x: atomic<i32> };
@group(0) @binding(0) var<storage, read_write> a: A;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
  return vec4f(p[i], 0., 1.);
}
@fragment fn main() -> @location(0) vec4<f32> { return vec4f(0.,0.,0.,1.); }
        "#;
        let err = Shader::new(wgsl).expect_err("expected error for atomic storage");
        match err {
            ShaderError::TypeMismatch(_) | ShaderError::ParseError(_) => {}
            _ => panic!("unexpected error kind: {:?}", err),
        }
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
var<push_constant> pc: PC;
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
var<push_constant> a: A;
var<push_constant> b: B;
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
                let (inner, _span, _) = data.first().unwrap();
                match inner {
                    UniformData::Struct((fields, _)) => {
                        // fields[0] should be items: Array
                        match &fields[0].2 {
                            UniformData::Array(items) => items.first().unwrap().2,
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
        use crate::mesh::{Vertex, VertexValue};
        mesh.add_vertices([
            Vertex::new([-0.5, -0.5, 0.0]),
            Vertex::new([0.5, -0.5, 0.0]),
            Vertex::new([0.0, 0.5, 0.0]),
        ]);
        // Add instance with wrong-typed "offset" (vec3 instead of vec2)
        mesh.add_instance(
            Vertex::new([0.0, 0.0]).set("offset", VertexValue::F32x3([0.0, 0.0, 0.0])),
        );

        let res = shader.add_mesh(&mesh);
        assert!(res.is_err());
        let s = format!("{}", res.unwrap_err());
        assert!(s.contains("Type mismatch for shader input 'offset'"));
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
}
