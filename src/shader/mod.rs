pub mod error;
use crate::{PassObject, Renderable};
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

mod features;
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

impl Default for Shader {
    fn default() -> Self {
        Self::new(DEFAULT_SHADER).expect("failed to create default shader")
    }
}

impl Shader {
    #[lsp_doc("docs/api/core/shader/new.md")]
    pub fn new(source: &str) -> Result<Self, ShaderError> {
        let object = Arc::new(input::load_shader(source)?);
        let pass = Arc::new(PassObject::from_shader_object(
            "Shader Default Pass",
            object.clone(),
        ));

        Ok(Self { pass, object })
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
}

impl Default for ShaderObject {
    fn default() -> Self {
        Self::new(DEFAULT_SHADER).expect("failed to create default shader object")
    }
}

impl ShaderObject {
    /// Create a Shader object from a WGSL source string.
    ///
    /// GLSL is also supported if you enable the `glsl` feature.
    /// Shadertoy-flavored GLSL is supported if the `shadertoy` feature is enabled.
    ///
    /// If the optional features are enabled,
    /// the constructor try to automatically detect the shader type and parse it accordingly.
    pub fn new(source: &str) -> Result<Self, ShaderError> {
        #[cfg(feature = "shadertoy")]
        if source.contains("void mainImage") {
            return ShaderObject::toy(source);
        }

        #[cfg(feature = "glsl")]
        if source.contains("void main") {
            return Self::glsl(DEFAULT_VERTEX_SHADER, source);
        }

        Self::wgsl(source)
    }

    /// Reflect the vertex entry-point inputs as (name, location, format).
    /// Returns only parameters with @location decorations; builtins are ignored.
    pub(crate) fn reflect_vertex_inputs(&self) -> Result<Vec<VertexInputDesc>, ShaderError> {
        // Find the vertex entry point (assume first if multiple; consistent with create_render_pipeline).
        let mut inputs: Vec<VertexInputDesc> = Vec::new();
        // Iterate entry points and collect from the vertex stage only once (first hit wins)
        for ep in self.module.entry_points.iter() {
            if ep.stage != naga::ShaderStage::Vertex {
                continue;
            }
            for arg in ep.function.arguments.iter() {
                // Only consider @location bindings
                let Some(binding) = arg.binding.as_ref() else {
                    continue;
                };
                let naga::Binding::Location { location, .. } = binding else {
                    continue;
                };
                let ty = &self.module.types[arg.ty];
                let format = naga_ty_to_vertex_format(ty)?;
                let name = arg
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
        inputs.sort_by_key(|d| d.location);
        Ok(inputs)
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
        })
    }

    /// Set a uniform value.
    pub fn set(&self, key: &str, value: impl Into<UniformData>) -> Result<(), ShaderError> {
        let mut storage = self.storage.write();
        storage.update(key, &value.into())
    }

    // getters
    /// List all the uniforms in the shader.
    pub fn list_uniforms(&self) -> Vec<String> {
        let storage = self.storage.read();
        storage.list()
    }

    /// List all available keys in the shader.
    pub fn list_keys(&self) -> Vec<String> {
        let storage = self.storage.read();
        storage.keys()
    }

    /// Get a uniform value as UniformData enum.
    pub(crate) fn get_uniform_data(&self, key: &str) -> Result<UniformData, ShaderError> {
        let storage = self.storage.read();
        let uniform = storage
            .get(key)
            .ok_or(ShaderError::UniformNotFound(key.into()))?;

        Ok(uniform.data.clone())
    }

    /// Get a uniform value as Uniform struct.
    pub(crate) fn get_uniform(&self, key: &str) -> Result<Uniform, ShaderError> {
        let storage = self.storage.read();
        let uniform = storage
            .get(key)
            .ok_or(ShaderError::UniformNotFound(key.into()))?;

        Ok(uniform.clone())
    }

    /// Tells weather the shader is a compute shader.
    pub(crate) fn is_compute(&self) -> bool {
        self.module
            .entry_points
            .iter()
            .any(|entry_point| entry_point.stage == naga::ShaderStage::Compute)
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
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        vec![self.pass.as_ref()]
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
        assert!((v - 0.5).abs() < 1e-6);
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
}
