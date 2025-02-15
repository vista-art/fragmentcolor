use crate::error::ShaderError;
use naga::{AddressSpace, Module};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub mod constants;
pub use constants::*;

pub mod compute;
pub use compute::*;

pub(crate) mod uniform;
pub(crate) use uniform::*;

mod storage;
use storage::*;

mod deserialize;

/// The hash of a shader source.
pub type ShaderHash = [u8; 32];

/// The Shader in FragmentColor is the blueprint of a Render Pipeline.
///
/// It automatically parses a WGSL shader and extracts its uniforms, buffers, and textures.
///
/// The user can set values for the uniforms and buffers, and then render the shader.
#[derive(Debug, Serialize)]
pub struct Shader {
    source: String,

    // Can be reconstructed from the source
    #[serde(skip_serializing)]
    pub(crate) hash: ShaderHash,
    #[serde(skip_serializing)]
    pub(crate) module: Module,
    #[serde(skip_serializing)]
    pub(crate) uniforms: HashMap<String, Uniform>,
    #[serde(skip_serializing)]
    pub(crate) storage: UniformStorage,
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
        #[cfg(feature = "shadertoy")]
        if source.contains("void mainImage") {
            return Shader::toy(source);
        }

        #[cfg(feature = "glsl")]
        if source.contains("void main") {
            return Self::glsl(DEFAULT_VERTEX_SHADER, source);
        }

        Self::wgsl(source)
    }

    /// Create a Shader object from a WGSL source.
    pub fn wgsl(source: &str) -> Result<Self, ShaderError> {
        let module = naga::front::wgsl::parse_str(source)?;
        let uniforms = parse_uniforms(&module)?;
        let storage = UniformStorage::new(&uniforms);

        let hash = hash(source);

        Ok(Self {
            source: source.to_string(),
            hash,
            module,
            uniforms,
            storage,
        })
    }

    pub fn set(&mut self, key: &str, value: impl Into<UniformData>) -> Result<(), ShaderError> {
        let (uniform_name, field_path) = parse_key(key);
        let uniform = self.get_uniform(&uniform_name)?;
        let value = value.into();

        let struct_field = get_struct_field(&uniform.data, &field_path)?;

        if std::mem::discriminant(&value) != std::mem::discriminant(&struct_field) {
            return Err(ShaderError::TypeMismatch(key.into()));
        }

        self.storage.update(key, &value);

        Ok(())
    }

    pub(crate) fn get_uniform(&self, uniform_name: &str) -> Result<&Uniform, ShaderError> {
        let uniform = self
            .uniforms
            .get(uniform_name)
            .ok_or(ShaderError::UniformNotFound(uniform_name.to_string()))?;

        Ok(uniform)
    }

    pub(crate) fn get_bytes(&self, key: &str) -> Result<&[u8], ShaderError> {
        let (uniform_name, field_path) = parse_key(key);
        let uniform = self.get_uniform(&uniform_name)?;
        let struct_field = get_struct_field(&uniform.data, &field_path)?;

        if std::mem::discriminant(&uniform.data) != std::mem::discriminant(&struct_field) {
            return Err(ShaderError::TypeMismatch(key.into()));
        }

        self.storage
            .get_bytes(key)
            .ok_or(ShaderError::UniformNotFound(key.into()))
    }

    pub unsafe fn get_as<T>(&self, key: &str) -> Result<T, ShaderError>
    where
        T: Copy,
    {
        let bytes = self.get_bytes(key)?;
        if bytes.len() != std::mem::size_of::<T>() {
            return Err(ShaderError::TypeMismatch(key.into()));
        }

        let value = unsafe { *(bytes.as_ptr() as *const T) };

        Ok(value)
    }
}

fn hash(source: &str) -> ShaderHash {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    let slice = hasher.finalize();

    slice.into()
}

fn parse_key(key: &str) -> (String, Vec<String>) {
    let mut parts = key.split('.');
    let uniform = parts.next().unwrap().to_string();
    let fields = parts.map(|s| s.to_string()).collect();
    (uniform, fields)
}

fn get_struct_field(ty: &UniformData, path: &[String]) -> Result<UniformData, ShaderError> {
    let mut current = ty;
    let mut offset = 0;
    for part in path {
        match current {
            UniformData::Struct(fields) => {
                let (field_offset, struct_field) = fields
                    .get(part)
                    .ok_or(ShaderError::FieldNotFound(part.clone()))?;
                offset += field_offset;
                current = struct_field;
            }
            _ => return Err(ShaderError::FieldNotFound(part.clone())),
        }
    }

    Ok(current.clone())
}

fn parse_uniforms(module: &Module) -> Result<HashMap<String, Uniform>, ShaderError> {
    let mut uniforms = HashMap::new();

    for (_, var) in module.global_variables.iter() {
        if var.space != AddressSpace::Uniform {
            continue;
        }

        let name = var
            .name
            .clone()
            .ok_or(ShaderError::ParseError("Unnamed uniform".into()))?;

        let binding = var
            .binding
            .as_ref()
            .ok_or(ShaderError::ParseError("Missing binding".into()))?;

        let ty = &module.types[var.ty];
        let size = ty.inner.size(module.to_ctx());

        uniforms.insert(
            name.clone(),
            Uniform {
                name,
                group: binding.group,
                binding: binding.binding,
                size,
                data: convert_type(module, ty)?,
            },
        );
    }

    Ok(uniforms)
}

#[cfg(feature = "glsl")]
impl Shader {
    /// Create a Shader object from a GLSL source pair (vertex and fragment shaders).
    pub fn glsl(vertex_source: &str, fragment_source: &str) -> Result<Self, ShaderError> {
        use naga::back::wgsl;
        use naga::front::glsl;
        use naga::valid::{
            Capabilities, ShaderStages, SubgroupOperationSet, ValidationFlags, Validator,
        };

        let mut parser = glsl::Frontend::default();
        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());

        let wgsl_vertex_source = {
            let vertex_module = parser.parse(
                &glsl::Options::from(naga::ShaderStage::Vertex),
                vertex_source,
            )?;
            let vertex_module_info = validator
                .subgroup_stages(ShaderStages::VERTEX)
                .subgroup_operations(SubgroupOperationSet::all())
                .validate(&vertex_module)?;

            wgsl::write_string(
                &vertex_module,
                &vertex_module_info,
                wgsl::WriterFlags::empty(),
            )?
            .replace("fn main", "fn vs_main")
        };

        let wgsl_fragment_source = {
            let fragment_module = parser.parse(
                &glsl::Options::from(naga::ShaderStage::Fragment),
                fragment_source,
            )?;
            let fragment_module_info = validator
                .subgroup_stages(ShaderStages::FRAGMENT)
                .subgroup_operations(SubgroupOperationSet::all())
                .validate(&fragment_module)?;

            wgsl::write_string(
                &fragment_module,
                &fragment_module_info,
                wgsl::WriterFlags::empty(),
            )?
            .replace("fn main", "fn fs_main")
        };

        Self::wgsl(&format!("{}\n{}", wgsl_vertex_source, wgsl_fragment_source))
    }

    #[cfg(feature = "shadertoy")]
    /// Create a Shader object from a Shadertoy-flavored GLSL source.
    pub fn toy(source: &str) -> Result<Self, ShaderError> {
        Self::glsl(
            DEFAULT_VERTEX_SHADER,
            &SHADERTOY_WRAPPER.replace("{{shader}}", source),
        )
    }
}

// @TODO - can't self-insert into an Arc and return it;
//         maybe we need to extract the shader state as metadata
//         into another struct and inject it into the render pass
//         or find another way to make it happen.
//         The goal is to let the user inject either a Shader or a Frame
//         into the Renderer.
// impl Renderable for Shader {
//     fn passes(&self) -> impl IntoIterator<Item = Pass> {
//         let mut render_pass = RenderPass::new();
//         render_pass.add_shader(Arc::new(self));

//         Some(Pass::Render(render_pass))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    const SHADER: &str = r#"
        struct VertexOutput {
            @builtin(position) coords: vec4<f32>,
        };

        @vertex
        fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
            let x = f32(i32(in_vertex_index) - 1);
            let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
            return vec4<f32>(x, y, 0.0, 1.0);
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
            let circle_pos = circle.position / resolution;
            let dist = distance(uv, circle_pos);
            let r = circle.radius / max(resolution.x, resolution.y);
            let circle_sdf = 1.0 - smoothstep(r - 0.001, r + 0.001, dist);
            return circle.color * circle_sdf;
        }
    "#;

    #[test]
    fn test_shader_should_parse_uniforms() {
        let shader = Shader::new(SHADER).unwrap();
        let mut uniforms = shader.uniforms.keys().collect::<Vec<_>>();
        uniforms.sort();
        assert_eq!(uniforms, vec!["circle", "resolution"]);
    }

    #[test]
    fn test_shader_should_set_uniform() {
        let mut shader = Shader::new(SHADER).unwrap();
        shader.set("circle.position", [0.5, 0.5]).unwrap();
        shader.set("circle.radius", 0.25).unwrap();
        shader.set("circle.color", [1.0, 0.0, 0.0, 1.0]).unwrap();
        shader.set("resolution", [800.0, 600.0]).unwrap();

        let position: [f32; 2] = unsafe { shader.get_as::<[f32; 2]>("circle.position").unwrap() };
        let radius: f32 = unsafe { shader.get_as::<f32>("circle.radius").unwrap() };
        let color: [f32; 4] = unsafe { shader.get_as::<[f32; 4]>("circle.color").unwrap() };
        let resolution: [f32; 2] = unsafe { shader.get_as::<[f32; 2]>("resolution").unwrap() };

        assert_eq!(position, [0.5, 0.5]);
        assert_eq!(radius, 0.25);
        assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(resolution, [800.0, 600.0]);
    }

    #[test]
    fn test_invalid_shader_should_return_error() {
        let result = Shader::new("invalid shader");
        assert!(result.is_err());
    }

    #[test]
    fn test_shader_serialization() {
        let shader = Shader::new(SHADER).unwrap();
        let serialized = serde_json::to_string(&shader).unwrap();

        let deserialized: Shader = serde_json::from_str(&serialized).unwrap();
        assert_eq!(shader.hash, deserialized.hash);
    }
}
