use crate::error::ShaderError;
use crate::uniform::{UniformLayout, UniformMetadata, UniformType};

// /// Lists all the ways you can input data into a GPU.
// ///
// /// Unused for now. In the first iteration, we will only use Uniforms.
// enum ShaderInputType {
//     VertexBuffer,
//     IndexBuffer,
//     StorageBuffer,
//     Uniform,
//     Texture,
//     StorageTexture,
//     Sampler,
//     Constant,
// }

// --- Core Data Structures ---
use naga::{AddressSpace, Module};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

pub struct Shader {
    source: String,
    hash: String,
    uniforms: HashMap<String, UniformMetadata>,
    values: BTreeMap<String, Vec<u8>>,
}

const DEFAULT_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

impl Default for Shader {
    fn default() -> Self {
        Self::new(DEFAULT_SHADER)
    }
}

// --- Shader Implementation ---
impl Shader {
    pub fn new(source: &str) -> Self {
        let hash = hash(source);
        let module = if let Ok(module) = naga::front::wgsl::parse_str(source) {
            module
        } else {
            log::warn!("Failed to parse shader source, using default shader");
            naga::front::wgsl::parse_str(DEFAULT_SHADER).unwrap()
        };

        let uniforms = if let Ok(uniforms) = Self::parse_uniforms(&module) {
            uniforms
        } else {
            log::warn!("Failed to parse uniforms, using empty uniforms");
            HashMap::new()
        };

        Self {
            source: source.to_string(),
            hash,
            uniforms,
            values: BTreeMap::new(),
        }
    }

    pub fn replace_shader(&mut self, source: &str) -> Result<(), ShaderError> {
        let hash = hash(source);
        let module = naga::front::wgsl::parse_str(source)?;
        let uniforms = Self::parse_uniforms(&module)?;

        self.source = source.to_string();
        self.hash = hash;
        self.uniforms = uniforms;
        self.values.clear();

        Ok(())
    }

    fn parse_uniforms(module: &Module) -> Result<HashMap<String, UniformMetadata>, ShaderError> {
        let mut uniforms = HashMap::new();

        for (_handle, var) in module.global_variables.iter() {
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
                UniformMetadata {
                    group: binding.group,
                    binding: binding.binding,
                    layout,
                },
            );
        }

        Ok(uniforms)
    }

    pub fn set<T: bytemuck::Pod>(&mut self, key: &str, value: T) -> Result<(), ShaderError> {
        let (uniform_name, field_path) = parse_key(key);
        let meta = self
            .uniforms
            .get(&uniform_name)
            .ok_or(ShaderError::UniformNotFound(uniform_name.clone()))?;

        let (offset, size) = get_field_offset(&meta.layout.ty, &field_path)?;
        let bytes = bytemuck::bytes_of(&value);

        if bytes.len() != size as usize {
            return Err(ShaderError::TypeMismatch(key.into()));
        }

        let buffer = self
            .values
            .entry(uniform_name.clone())
            .or_insert_with(|| vec![0; meta.layout.size as usize]);

        buffer[offset as usize..(offset + size) as usize].copy_from_slice(bytes);
        Ok(())
    }

    pub fn append(&mut self, source: &str) {
        self.source.push_str(source);
    }

    pub fn print(&self) {
        println!("Shader: {}", self.source);
    }
}

fn hash(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn parse_key(key: &str) -> (String, Vec<String>) {
    let mut parts = key.split('.');
    let uniform = parts.next().unwrap().to_string();
    let fields = parts.map(|s| s.to_string()).collect();
    (uniform, fields)
}

fn get_field_offset(ty: &UniformType, path: &[String]) -> Result<(u32, u32), ShaderError> {
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
    let size = match current {
        UniformType::Float => 4,
        UniformType::Vec2 => 8,
        UniformType::Vec3 => 12,
        UniformType::Vec4 => 16,
        _ => return Err(ShaderError::TypeMismatch("Unsupported type".into())),
    };
    Ok((offset, size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader() {
        let shader = Shader::new("shader");
        assert_eq!(shader.source, "shader");
    }
}
