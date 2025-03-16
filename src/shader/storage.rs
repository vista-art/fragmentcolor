use std::collections::HashMap;

use crate::ShaderError;

use super::{Uniform, UniformData};

#[derive(Debug, Clone)]
pub(crate) struct UniformStorage {
    pub(crate) uniform_bytes: Vec<u8>,
    pub(crate) uniforms: HashMap<String, (u32, u32, Uniform)>, // (offset, size, original data)
}

impl UniformStorage {
    /// Create a new UniformStorage from a HashMap of key->Uniform
    pub(crate) fn new(uniforms: &HashMap<String, Uniform>) -> Self {
        let mut storage = Self {
            uniform_bytes: Vec::new(),
            uniforms: HashMap::new(),
        };

        storage.extend(uniforms);

        storage
    }

    /// Extend the block with a HashMap of key->UniformData
    fn extend(&mut self, uniforms: &HashMap<String, Uniform>) {
        for uniform in uniforms.values() {
            self.add_uniform(uniform);
        }
    }

    /// Add a single uniform to the storage.
    ///
    /// Uniforms are cached by name. A Struct uniform will be flattened as bytes.
    ///
    /// We will index both the uniform name and its fields with the dot notation.
    /// For example, for a struct uniform named `light` with fields `position` and `color`,
    /// we will index the uniform as `light` and the fields as `light.position` and `light.color`.
    pub(crate) fn add_uniform(&mut self, uniform: &Uniform) {
        let uniform_offset = self.uniform_bytes.len() as u32;
        self.uniform_bytes.extend(uniform.data.to_bytes());
        self.uniforms.insert(
            uniform.name.clone(),
            (uniform_offset, uniform.data.size(), uniform.clone()),
        );

        // If the Uniform is a struct, we also index its fields to allow granular access
        if let UniformData::Struct((fields, _)) = &uniform.data {
            for (field_offset, field_name, field) in fields {
                let key = format!("{}.{}", uniform.name, field_name);
                self.uniforms.insert(
                    key.clone(),
                    (
                        uniform_offset + field_offset,
                        field.size(),
                        Uniform {
                            name: key,
                            group: uniform.group,
                            binding: uniform.binding,
                            data: field.clone(),
                        },
                    ),
                );
            }
        }
    }

    /// Update a single uniform
    pub fn update(&mut self, key: &str, value: &UniformData) -> Result<(), ShaderError> {
        if let Some((offset, size, uniform)) = self.uniforms.get_mut(key) {
            if std::mem::discriminant(value) != std::mem::discriminant(&uniform.data) {
                return Err(ShaderError::TypeMismatch(key.into()));
            }

            uniform.data = value.clone();

            // update raw uniform bytes
            let value_bytes = value.to_bytes();
            if value_bytes.len() == *size as usize {
                let start = *offset as usize;
                let end = start + value_bytes.len();
                self.uniform_bytes[start..end].copy_from_slice(&value_bytes);
            }

            Ok(())
        } else {
            Err(ShaderError::UniformNotFound(key.into()))
        }
    }

    /// List all top-level uniforms
    pub fn list(&self) -> Vec<String> {
        self.uniforms
            .keys()
            .filter(|k| !k.contains('.'))
            .cloned()
            .collect()
    }

    /// List all uniform keys
    pub fn keys(&self) -> Vec<String> {
        self.uniforms.keys().cloned().collect()
    }

    /// Get a uniform by key
    pub fn get(&self, key: &str) -> Option<&Uniform> {
        if let Some((_, _, uniform)) = self.uniforms.get(key) {
            Some(uniform)
        } else {
            None
        }
    }

    /// Get a uniform as bytes by key
    pub fn get_bytes(&self, key: &str) -> Option<&[u8]> {
        self.uniforms.get(key).map(|(offset, size, _)| {
            let start = *offset as usize;
            let end = start + *size as usize;
            &self.uniform_bytes[start..end]
        })
    }
}
