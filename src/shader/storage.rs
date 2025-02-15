use std::collections::HashMap;

use super::{Uniform, UniformData};

#[derive(Debug, Clone)]
pub(crate) struct UniformStorage {
    pub(crate) uniform_bytes: Vec<u8>,
    pub(crate) offsets: HashMap<String, (u32, u32)>, // (offset, size)
}

impl UniformStorage {
    pub(crate) fn new(uniforms: &HashMap<String, Uniform>) -> Self {
        let mut storage = Self {
            uniform_bytes: Vec::new(),
            offsets: HashMap::new(),
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
        let offset = self.uniform_bytes.len() as u32;
        self.uniform_bytes.extend(uniform.data.to_bytes());
        self.offsets
            .insert(uniform.name.clone(), (offset, uniform.data.size()));

        // If the Uniform is a struct, we also index its fields to allow granular access
        if let UniformData::Struct(fields) = &uniform.data {
            for (field_name, (offset, field)) in fields.iter() {
                self.offsets.insert(
                    format!("{}.{}", uniform.name, field_name),
                    (offset + offset, field.size()),
                );
            }
        }
    }

    // Update a single uniform by copying into the existing data slice
    pub fn update(&mut self, key: &str, uniform: &UniformData) {
        if let Some((offset, size)) = self.offsets.get(key) {
            let offset = *offset as usize;
            let size = *size as usize;
            let raw = uniform.to_bytes();
            if raw.len() == size {
                self.uniform_bytes[offset..offset + size].copy_from_slice(&raw);
            }
        }
    }

    // get uniform as bytes
    pub fn get_bytes(&self, key: &str) -> Option<&[u8]> {
        if let Some((offset, size)) = self.offsets.get(key) {
            Some(&self.uniform_bytes[*offset as usize..*offset as usize + *size as usize])
        } else {
            None
        }
    }
}
