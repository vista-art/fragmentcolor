use std::collections::HashMap;

use crate::ShaderError;

use super::{Uniform, UniformData};

#[derive(Debug, Clone)]
pub(crate) struct UniformStorage {
    pub(crate) uniform_bytes: Vec<u8>,
    pub(crate) uniforms: HashMap<String, (u32, u32, Uniform)>, // (offset, size, original data)
    // CPU-side blobs for storage buffers: root name -> raw bytes
    pub(crate) storage_blobs: HashMap<String, Vec<u8>>,
}

impl UniformStorage {
    /// Create a new UniformStorage from a HashMap of key->Uniform
    pub(crate) fn new(uniforms: &HashMap<String, Uniform>) -> Self {
        let mut storage = Self {
            uniform_bytes: Vec::new(),
            uniforms: HashMap::new(),
            storage_blobs: HashMap::new(),
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
        // For storage buffers, we keep a separate CPU blob and do not write to uniform_bytes.
        // For other value types, append to uniform_bytes and index by offset.
        let mut base_offset = self.uniform_bytes.len() as u32;
        let is_storage = matches!(uniform.data, UniformData::Storage(_));
        if !is_storage {
            self.uniform_bytes.extend(uniform.data.to_bytes());
        } else {
            base_offset = 0;
        }
        self.uniforms.insert(
            uniform.name.clone(),
            (base_offset, uniform.data.size(), uniform.clone()),
        );

        // If the Uniform is a struct, we also index its fields to allow granular access
        if let UniformData::Struct((fields, _)) = &uniform.data {
            for (field_offset, field_name, field) in fields {
                let key = format!("{}.{}", uniform.name, field_name);
                self.uniforms.insert(
                    key.clone(),
                    (
                        base_offset + field_offset,
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

        // Storage buffers have their own GPU buffer; create a CPU blob and index fields with offsets
        if let UniformData::Storage(data) = &uniform.data
            && let Some((inner, span, _)) = data.iter().next()
        {
            // Create blob if missing
            self.storage_blobs
                .entry(uniform.name.clone())
                .or_insert_with(|| vec![0u8; *span as usize]);

            // Index fields with their declared offsets relative to the storage blob
            if let UniformData::Struct((fields, _)) = inner {
                for (field_offset, field_name, field) in fields {
                    let key = format!("{}.{}", uniform.name, field_name);
                    self.uniforms.insert(
                        key.clone(),
                        (
                            *field_offset,
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
            // Also ensure the top-level storage buffer entry is present with offset 0
            self.uniforms.insert(
                uniform.name.clone(),
                (
                    0,
                    *span,
                    Uniform {
                        name: uniform.name.clone(),
                        group: uniform.group,
                        binding: uniform.binding,
                        data: uniform.data.clone(),
                    },
                ),
            );
        }
    }

    /// Update a single uniform
    pub fn update(&mut self, key: &str, value: &UniformData) -> Result<(), ShaderError> {
        if let Some((offset, size, uniform)) = self.uniforms.get_mut(key) {
            // Allow updating Texture with TextureMeta (id + naga metadata) and preserve shader metadata if caller passed id-only
            match (&uniform.data, value) {
                (UniformData::Texture(existing), UniformData::Texture(incoming)) => {
                    let merged = if incoming.id.0 != 0 {
                        crate::texture::TextureMeta {
                            id: incoming.id.clone(),
                            dim: existing.dim,
                            arrayed: existing.arrayed,
                            class: existing.class,
                        }
                    } else {
                        existing.clone()
                    };
                    uniform.data = UniformData::Texture(merged);
                }
                _ => {
                    if std::mem::discriminant(value) != std::mem::discriminant(&uniform.data) {
                        return Err(ShaderError::TypeMismatch(key.into()));
                    }
                    uniform.data = value.clone();
                }
            }

            // Write into CPU-side blobs
            // 1) Uniform buffer bytes for classic uniforms
            // Determine root storage name first so we can avoid writing storage fields into uniform_bytes.
            let root = key.split('.').next().unwrap_or(key);
            let is_storage_root = self.storage_blobs.contains_key(root);
            if !is_storage_root
                && !matches!(
                    value,
                    UniformData::Texture(_) | UniformData::Sampler(_) | UniformData::Storage(_)
                )
            {
                let value_bytes = value.to_bytes();
                if value_bytes.len() == *size as usize {
                    let start = *offset as usize;
                    let end = start + value_bytes.len();
                    self.uniform_bytes[start..end].copy_from_slice(&value_bytes);
                }
            }
            // 2) Storage buffer blobs
            if let Some(blob) = self.storage_blobs.get_mut(root) {
                match value {
                    UniformData::Storage(data) if key == root => {
                        if let Some((inner, span, _)) = data.iter().next() {
                            // Write full storage blob from inner
                            let data = inner.to_bytes();
                            let n = (*span as usize).min(data.len());
                            if blob.len() < *span as usize {
                                blob.resize(*span as usize, 0);
                            }
                            blob[0..n].copy_from_slice(&data[0..n]);
                            if n < blob.len() {
                                for b in blob[n..].iter_mut() {
                                    *b = 0;
                                }
                            }
                        }
                    }
                    _ => {
                        // Nested field: copy at offset
                        let start = *offset as usize;
                        let data = value.to_bytes();
                        let end = start.saturating_add(data.len());
                        if end <= blob.len() {
                            blob[start..end].copy_from_slice(&data);
                        }
                    }
                }
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
        if let Some((offset, size, uniform)) = self.uniforms.get(key) {
            // If this key corresponds to a storage buffer (root or nested), slice from the storage blob
            let root = if matches!(uniform.data, UniformData::Storage(_)) {
                key
            } else {
                key.split('.').next().unwrap_or(key)
            };

            if let Some(blob) = self.storage_blobs.get(root) {
                let start = *offset as usize;
                let end = start + *size as usize;
                return Some(&blob[start..end]);
            }

            // Fallback to uniform_bytes for classic uniforms
            let start = *offset as usize;
            let end = start + *size as usize;
            return Some(&self.uniform_bytes[start..end]);
        }

        None
    }
}
