use crate::error::ShaderError;
use naga::{AddressSpace, Module};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

pub mod compute;
pub use compute::*;

pub mod uniform;
pub use uniform::*;

mod input;

#[derive(Debug, Clone, PartialEq)]
pub enum ShaderValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    IVec2([i32; 2]),
    IVec3([i32; 3]),
    IVec4([i32; 4]),
    UInt(u32),
    UVec2([u32; 2]),
    UVec3([u32; 3]),
    UVec4([u32; 4]),
    Mat2([[f32; 2]; 2]),
    Mat3([[f32; 3]; 3]),
    Mat4([[f32; 4]; 4]),
    Texture(u64),
}

impl ShaderValue {
    fn data_type(&self) -> UniformType {
        match self {
            Self::Float(_) => UniformType::Float,
            Self::Vec2(_) => UniformType::Vec2,
            Self::Vec3(_) => UniformType::Vec3,
            Self::Vec4(_) => UniformType::Vec4,
            Self::Int(_) => UniformType::Int,
            Self::IVec2(_) => UniformType::IVec2,
            Self::IVec3(_) => UniformType::IVec3,
            Self::IVec4(_) => UniformType::IVec4,
            Self::UInt(_) => UniformType::UInt,
            Self::UVec2(_) => UniformType::UVec2,
            Self::UVec3(_) => UniformType::UVec3,
            Self::UVec4(_) => UniformType::UVec4,
            Self::Mat2(_) => UniformType::Mat2,
            Self::Mat3(_) => UniformType::Mat3,
            Self::Mat4(_) => UniformType::Mat4,
            Self::Texture(_) => UniformType::Texture,
        }
    }

    fn to_bytes(&self) -> &[u8] {
        match self {
            Self::Float(v) => bytemuck::bytes_of(v),
            Self::Int(v) => bytemuck::bytes_of(v),
            Self::UInt(v) => bytemuck::bytes_of(v),
            Self::Vec2(v) => bytemuck::cast_slice(v),
            Self::Vec3(v) => bytemuck::cast_slice(v),
            Self::Vec4(v) => bytemuck::cast_slice(v),
            Self::IVec2(v) => bytemuck::cast_slice(v),
            Self::IVec3(v) => bytemuck::cast_slice(v),
            Self::IVec4(v) => bytemuck::cast_slice(v),
            Self::UVec2(v) => bytemuck::cast_slice(v),
            Self::UVec3(v) => bytemuck::cast_slice(v),
            Self::UVec4(v) => bytemuck::cast_slice(v),
            Self::Mat2(v) => bytemuck::cast_slice(v.as_slice()),
            Self::Mat3(v) => bytemuck::cast_slice(v.as_slice()),
            Self::Mat4(v) => bytemuck::cast_slice(v.as_slice()),
            Self::Texture(h) => bytemuck::bytes_of(h),
        }
    }
}

const DEFAULT_VERTEX_SHADER: &str = r#"
    #version 450

    layout(location = 0) in vec2 position;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;

const SHADERTOY_WRAPPER: &str = r#"
    uniform vec3      iResolution;           // viewport resolution (in pixels)
    uniform float     iTime;                 // shader playback time (in seconds)
    uniform float     iTimeDelta;            // render time (in seconds)
    uniform float     iFrameRate;            // shader frame rate
    uniform int       iFrame;                // shader playback frame
    uniform float     iChannelTime[4];       // channel playback time (in seconds)
    uniform vec3      iChannelResolution[4]; // channel resolution (in pixels)
    uniform vec4      iMouse;                // mouse pixel coords. xy: current (if MLB down)

    void main() {
        vec4 fragColor;
        mainImage(fragColor, gl_FragCoord.xy);
        gl_FragColor = fragColor;
    }

    {{shader}}
"#;

/// The Shader object in FragmentColor is the blueprint of a shader program
/// and the public interface represents a Render Pipeline.
///
/// It automatically parses a WGSL shader and extracts its uniforms, buffers, and textures.
///
/// The user can set values for the uniforms and buffers, and then render the shader.
#[derive(Debug, Serialize)]
pub struct Shader {
    source: String,

    // Those can be reconstructed from the source
    #[serde(skip_serializing)]
    pub(crate) hash: [u8; 64],
    #[serde(skip_serializing)]
    pub(crate) module: Module,
    #[serde(skip_serializing)]
    pub(crate) uniforms: HashMap<String, Uniform>,

    // @TODO update the test; the values should be included in the serialization
    #[serde(skip_serializing)]
    values: BTreeMap<String, ShaderValue>,
}

impl<'de> Deserialize<'de> for Shader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Source, // we only need the source to rebuild the Struct
        }

        struct ShaderVisitor;

        impl<'de> Visitor<'de> for ShaderVisitor {
            type Value = Shader;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Shader")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut source: Option<String> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Source => {
                            if source.is_some() {
                                return Err(de::Error::duplicate_field("source"));
                            }
                            source = Some(map.next_value()?);
                        }
                    }
                }
                let source = source.ok_or_else(|| de::Error::missing_field("source"))?;
                Shader::new(&source).map_err(de::Error::custom)
            }
        }

        const FIELDS: &[&str] = &["source"];
        deserializer.deserialize_struct("Shader", FIELDS, ShaderVisitor)
    }
}

// --- Shader Implementation ---
impl Shader {
    pub fn hash(&self) -> [u8; 64] {
        self.hash
    }

    /// Create a Shader object from a source string.
    ///
    /// The source string can be in WGSL, GLSL, or Shadertoy-flavored GLSL.
    /// The function will automatically detect the shader type and parse it accordingly.
    pub fn new(source: &str) -> Result<Self, ShaderError> {
        if source.contains("void mainImage") {
            Shader::toy(source)
        } else if source.contains("void main") {
            Self::glsl(DEFAULT_VERTEX_SHADER, source)
        } else {
            Self::wgsl(source)
        }
    }

    /// Create a Shader object from a WGSL source.
    pub fn wgsl(source: &str) -> Result<Self, ShaderError> {
        let module = naga::front::wgsl::parse_str(source)?;

        let uniforms = if let Ok(uniforms) = Self::parse_uniforms(&module) {
            uniforms
        } else {
            HashMap::new()
        };

        let hash = hash(source);

        Ok(Self {
            source: source.to_string(),
            hash,
            module,
            uniforms,
            values: BTreeMap::new(),
        })
    }

    /// Create a Shader object from a Shadertoy-flavored GLSL source.
    pub fn toy(source: &str) -> Result<Self, ShaderError> {
        Self::glsl(
            DEFAULT_VERTEX_SHADER,
            &SHADERTOY_WRAPPER.replace("{{shader}}", source),
        )
    }

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

    pub fn replace(&mut self, source: &str) -> Result<(), ShaderError> {
        let module = naga::front::wgsl::parse_str(source)?;
        let uniforms = Self::parse_uniforms(&module)?;
        let hash = hash(source);

        self.source = source.to_string();
        self.hash = hash;
        self.uniforms = uniforms;
        self.values.clear();

        Ok(())
    }

    pub fn set(&mut self, key: &str, value: impl Into<ShaderValue>) -> Result<(), ShaderError> {
        let value = value.into();

        let (uniform_name, field_path) = parse_key(key);
        let meta = self
            .uniforms
            .get(&uniform_name)
            .ok_or(ShaderError::UniformNotFound(uniform_name.clone()))?;

        let ty = get_field_type(&meta.layout.ty, &field_path)?;

        if value.data_type() != ty {
            return Err(ShaderError::TypeMismatch(key.into()));
        }

        self.values.insert(key.into(), value);

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<&[u8], ShaderError> {
        let (uniform_name, field_path) = parse_key(key);
        let meta = self
            .uniforms
            .get(&uniform_name)
            .ok_or(ShaderError::UniformNotFound(uniform_name.clone()))?;

        let ty = get_field_type(&meta.layout.ty, &field_path)?;
        let value = self
            .values
            .get(&uniform_name)
            .ok_or(ShaderError::UniformNotFound(uniform_name.clone()))?;

        if value.data_type() != ty {
            return Err(ShaderError::TypeMismatch(key.into()));
        }

        Ok(value.to_bytes())
    }

    pub unsafe fn get_as<T>(&self, key: &str) -> Result<T, ShaderError>
    where
        T: Copy,
    {
        let bytes = self.get(key)?;
        if bytes.len() != std::mem::size_of::<T>() {
            return Err(ShaderError::TypeMismatch(key.into()));
        }

        let value = unsafe { *(bytes.as_ptr() as *const T) };

        Ok(value)
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

            let layout = UniformLayout::from_naga_type(module, &module.types[var.ty])?;

            uniforms.insert(
                name,
                Uniform {
                    group: binding.group,
                    binding: binding.binding,
                    layout,
                },
            );
        }

        Ok(uniforms)
    }
}

fn hash(source: &str) -> [u8; 64] {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    hasher.finalize().as_slice().try_into().unwrap()
}

fn parse_key(key: &str) -> (String, Vec<String>) {
    let mut parts = key.split('.');
    let uniform = parts.next().unwrap().to_string();
    let fields = parts.map(|s| s.to_string()).collect();
    (uniform, fields)
}

fn get_field_type(ty: &UniformType, path: &[String]) -> Result<UniformType, ShaderError> {
    let mut current = ty;
    let mut offset = 0;
    for part in path {
        match current {
            UniformType::Struct(fields) => {
                let (field_offset, field_type) = fields
                    .get(part)
                    .ok_or(ShaderError::FieldNotFound(part.clone()))?;
                offset += field_offset;
                current = field_type;
            }
            _ => return Err(ShaderError::FieldNotFound(part.clone())),
        }
    }

    Ok(current.clone())
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
        struct Circle {
            position: vec2<f32>,
            radius: f32,
            color: vec4<f32>,
        }

        @group(0) @binding(0)
        var<uniform> circle: Circle;

        @group(0) @binding(1) var<uniform> resolution: vec2<f32>;

        @fragment
        fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
            let uv = pos.xy / resolution;
            let circle_pos = position / resolution;
            let dist = distance(uv, circle_pos);
            let r = radius / max(resolution.x, resolution.y);
            let circle = 1.0 - smoothstep(r - 0.001, r + 0.001, dist);
            return color * circle;
        }
    "#;

    #[test]
    fn test_shader_should_parse_uniforms() {
        let shader = Shader::new(SHADER).unwrap();
        let uniforms = shader.uniforms.keys().collect::<Vec<_>>();
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

        // it should have only the Source field
        assert_eq!(
            serialized,
            r#"{"source":"struct Circle {\n    position: vec2<f32>,\n    radius: f32,\n    color: vec4<f32>,\n}\n\n@group(0) @binding(0)\nvar<uniform> circle: Circle;\n\n@group(0) @binding(1) var<uniform> resolution: vec2<f32;\n\n@fragment\nfn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {\n    let uv = pos.xy / resolution;\n    let circle_pos = position / resolution;\n    let dist = distance(uv, circle_pos);\n    let r = radius / max(resolution.x, resolution.y);\n    let circle = 1.0 - smoothstep(r - 0.001, r + 0.001, dist);\n    return color * circle;\n}\n"}"#
        );

        let deserialized: Shader = serde_json::from_str(&serialized).unwrap();
        assert_eq!(shader.hash(), deserialized.hash());
    }
}
