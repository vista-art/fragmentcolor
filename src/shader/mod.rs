use crate::error::ShaderError;
use crate::{PassObject, Renderable};
use naga::{
    valid::{Capabilities, ValidationFlags, Validator},
    AddressSpace, Module,
};
use parking_lot::RwLock;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(python)]
use pyo3::prelude::*;

pub mod constants;
mod features;
pub use constants::*;
pub(crate) mod uniform;
pub(crate) use uniform::*;
mod deserialize;
mod input;
mod storage;
use storage::*;

/// The hash of a shader source.
pub type ShaderHash = [u8; 32];

#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug)]
/// The Shader in FragmentColor is the blueprint of a Render Pipeline.
///
/// It automatically parses a WGSL shader and extracts its uniforms, buffers, and textures.
///
/// The user can set values for the uniforms and buffers, and then render the shader.
pub struct Shader {
    pub(crate) pass: Arc<PassObject>,
    pub(crate) object: Arc<ShaderObject>,
}

impl Shader {
    /// Create a Shader object from a WGSL source string.
    ///
    /// GLSL is also supported if you enable the `glsl` feature.
    /// Shadertoy-flavored GLSL is supported if the `shadertoy` feature is enabled.
    ///
    /// If the optional features are enabled,
    /// the constructor try to automatically detect the shader type and parse it accordingly.
    pub fn new(source: &str) -> Result<Self, ShaderError> {
        let object = Arc::new(input::load_shader(source)?);
        let pass = Arc::new(PassObject::from_shader_object(
            "Shader Default Pass",
            object.clone(),
        ));

        Ok(Self { pass, object })
    }

    /// Set a uniform value.
    pub fn set(&self, key: &str, value: impl Into<UniformData>) -> Result<(), ShaderError> {
        self.object.set(key, value)
    }

    /// Get a uniform value.
    pub fn get<T: From<UniformData>>(&self, key: &str) -> Result<T, ShaderError> {
        Ok(self.object.get_uniform_data(key)?.into())
    }

    /// List all the top-level uniforms in the shader.
    pub fn list_uniforms(&self) -> Vec<String> {
        self.object.list_uniforms()
    }

    /// List all available keys in the shader.
    /// This includes all the uniforms and their fields.
    pub fn list_keys(&self) -> Vec<String> {
        self.object.list_keys()
    }
}

/// FragmentColor's Shader internal implementation.
///
/// The ShaderObject is wrapped in an Arc and managed by the Shader struct.
/// This allows it to be shared between multiple passes and render pipelines.
#[derive(Debug, Serialize)]
pub(crate) struct ShaderObject {
    pub(crate) source: String,

    // Can be reconstructed from the source
    #[serde(skip_serializing)]
    pub(crate) hash: ShaderHash,
    #[serde(skip_serializing)]
    pub(crate) module: Module,
    #[serde(skip_serializing)]
    pub(crate) storage: RwLock<UniformStorage>,
    #[serde(skip_serializing)]
    pub(crate) total_bytes: u64,
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

    /// Create a Shader object from a WGSL source.
    pub fn wgsl(source: &str) -> Result<Self, ShaderError> {
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
        let module = naga::front::wgsl::parse_str(source)?;
        validator.validate(&module)?;

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
            source: source.to_string(),
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
}

// getters
impl ShaderObject {
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
        if variable.space != AddressSpace::Uniform {
            continue;
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

        uniforms.insert(
            uniform_name.clone(),
            Uniform {
                name: uniform_name,
                group: binding.group,
                binding: binding.binding,
                data: convert_type(module, ty)?,
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_shader_serialization() {
        let shader = ShaderObject::new(SHADER).unwrap();
        let serialized = serde_json::to_string(&shader).unwrap();

        let deserialized: ShaderObject = serde_json::from_str(&serialized).unwrap();
        assert_eq!(shader.hash, deserialized.hash);
    }
}
