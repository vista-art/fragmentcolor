use std::collections::{HashMap, HashSet};

use crate::ShaderError;

use super::{Uniform, UniformData};

#[derive(Debug, Clone)]
pub(crate) struct UniformStorage {
    pub(crate) uniform_bytes: Vec<u8>,
    pub(crate) uniforms: HashMap<String, (u32, u32, Uniform)>, // (offset, size, original data)
    // CPU-side blobs for storage buffers: root name -> raw bytes
    pub(crate) storage_blobs: HashMap<String, Vec<u8>>,
    // Track which storage roots have been modified on CPU since last GPU upload
    pub(crate) storage_dirty: HashSet<String>,
    // CPU-side blobs for push constants: root name -> raw bytes
    pub(crate) push_blobs: HashMap<String, Vec<u8>>,
}

impl UniformStorage {
    /// Create a new UniformStorage from a HashMap of key->Uniform
    pub(crate) fn new(uniforms: &HashMap<String, Uniform>) -> Self {
        let mut storage = Self {
            uniform_bytes: Vec::new(),
            uniforms: HashMap::new(),
            storage_blobs: HashMap::new(),
            storage_dirty: HashSet::new(),
            push_blobs: HashMap::new(),
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
        // For storage buffers AND push constants, we keep a separate CPU blob
        // and do not write to uniform_bytes.
        // For other value types, append to uniform_bytes and index by offset.
        let mut base_offset = self.uniform_bytes.len() as u32;

        let is_storage = matches!(uniform.data, UniformData::Storage(_));
        let is_push = matches!(uniform.data, UniformData::PushConstant(_));
        if is_storage || is_push {
            base_offset = self.uniform_bytes.len() as u32;
        } else {
            self.uniform_bytes.extend(uniform.data.to_bytes());
        }

        // Determine span for indexing; push constants report 0 via size(),
        // so compute explicitly.
        let mut index_size = uniform.data.size();
        if let UniformData::PushConstant(data) = &uniform.data
            && let Some((_uniform, span)) = data.first()
        {
            index_size = *span;
        }

        self.uniforms.insert(
            uniform.name.clone(),
            (base_offset, index_size, uniform.clone()),
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

        // Push constants: maintain a per-root CPU blob similar to storage
        if let UniformData::PushConstant(data) = &uniform.data
            && let Some((inner, span)) = data.iter().next()
        {
            let span = *span;
            self.push_blobs
                .entry(uniform.name.clone())
                .or_insert_with(|| vec![0u8; span as usize]);

            // Index fields for direct access if it's a struct
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
        }
    }

    /// Directly replace the CPU-side bytes of a storage buffer root.
    /// - root must be a top-level storage buffer name (e.g., "particles").
    /// - bytes will be copied into the blob up to the declared span; remaining region is zeroed.
    pub fn set_storage_bytes(&mut self, root: &str, bytes: &[u8]) -> Result<(), ShaderError> {
        if let Some(blob) = self.storage_blobs.get_mut(root) {
            let span = blob.len();
            let n = span.min(bytes.len());
            blob[..n].copy_from_slice(&bytes[..n]);
            if n < span {
                for b in blob[n..].iter_mut() {
                    *b = 0;
                }
            }
            // Mark root dirty for next GPU upload
            self.storage_dirty.insert(root.to_string());
            Ok(())
        } else {
            Err(ShaderError::UniformNotFound(root.into()))
        }
    }

    /// Directly replace the CPU-side bytes of a push-constant root.
    /// - root must be a top-level push-constant name (e.g., "pc").
    /// - bytes will be copied into the blob up to the declared span; remaining region is zeroed.
    pub fn set_push_bytes(&mut self, root: &str, bytes: &[u8]) -> Result<(), ShaderError> {
        if let Some(blob) = self.push_blobs.get_mut(root) {
            let span = blob.len();
            let n = span.min(bytes.len());
            blob[..n].copy_from_slice(&bytes[..n]);
            if n < span {
                for b in blob[n..].iter_mut() {
                    *b = 0;
                }
            }
            Ok(())
        } else {
            Err(ShaderError::UniformNotFound(root.into()))
        }
    }

    /// Update a single uniform
    pub fn update(&mut self, key: &str, value: &UniformData) -> Result<(), ShaderError> {
        // Fast path: exact key exists in the index
        if let Some((offset, size, uniform)) = self.uniforms.get_mut(key) {
            // Special-case: storage root receiving raw bytes
            let root = key.split('.').next().unwrap_or(key).to_string();
            let is_storage_root = matches!(uniform.data, UniformData::Storage(_)) && root == *key;
            if is_storage_root && let UniformData::Bytes(b) = value {
                self.set_storage_bytes(&root, b)?;
                return Ok(());
            }
            // Special-case: push root receiving raw bytes
            let is_push_root = matches!(uniform.data, UniformData::PushConstant(_)) && root == *key;
            if is_push_root && let UniformData::Bytes(b) = value {
                self.set_push_bytes(&root, b)?;
                return Ok(());
            }

            // Allow updating Texture with TextureMeta (id + naga metadata) and preserve shader metadata if caller passed id-only
            match (&uniform.data, value) {
                (UniformData::Texture(existing), UniformData::Texture(incoming)) => {
                    let merged = if incoming.id.0 != 0 {
                        crate::texture::TextureMeta {
                            id: incoming.id,
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
            // Determine root names first so we can avoid writing storage/push fields into uniform_bytes.
            let root = key.split('.').next().unwrap_or(key);
            let is_storage_root = self.storage_blobs.contains_key(root);
            let is_push_constant_root = self.push_blobs.contains_key(root);
            if !is_storage_root
                && !is_push_constant_root
                && !matches!(
                    value,
                    UniformData::Texture(_)
                        | UniformData::Sampler(_)
                        | UniformData::Storage(_)
                        | UniformData::PushConstant(_)
                )
            {
                let value_bytes = value.to_bytes();
                if value_bytes.len() == *size as usize {
                    let start = *offset as usize;
                    let end = start + value_bytes.len();
                    if end <= self.uniform_bytes.len() {
                        self.uniform_bytes[start..end].copy_from_slice(&value_bytes);
                    }
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
                            // Mark dirty
                            self.storage_dirty.insert(root.to_string());
                        }
                    }
                    _ => {
                        // Nested field: copy at offset
                        let start = *offset as usize;
                        let data = value.to_bytes();
                        let end = start.saturating_add(data.len());
                        if end <= blob.len() {
                            blob[start..end].copy_from_slice(&data);
                            // Mark dirty for nested update as well
                            self.storage_dirty.insert(root.to_string());
                        }
                    }
                }
            }

            // 3) Push constant blobs (root or nested field)
            if let Some(blob) = self.push_blobs.get_mut(root) {
                match value {
                    UniformData::PushConstant(data) if key == root => {
                        if let Some((inner, span)) = data.iter().next() {
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
                        // Nested field update
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
            // Slow path: support keys with array indices like "data.arr[3].field" or "u[2]"
            let (root, parts) = parse_key(key)?;
            // Find the root entry (top-level uniform variable)
            let (root_offset, _root_size, root_uniform) = self
                .uniforms
                .get(&root)
                .cloned()
                .ok_or_else(|| ShaderError::UniformNotFound(root.clone()))?;

            let is_storage = matches!(root_uniform.data, UniformData::Storage(_));
            let is_push = matches!(root_uniform.data, UniformData::PushConstant(_));

            let (rel_ofs, leaf_sz) = if is_storage {
                // For storage buffers, traverse the inner shape
                if let UniformData::Storage(data) = &root_uniform.data {
                    if let Some((inner, _span, _)) = data.first() {
                        compute_offset(inner, &parts, key)?
                    } else {
                        return Err(ShaderError::InvalidKey(key.into()));
                    }
                } else {
                    return Err(ShaderError::InvalidKey(key.into()));
                }
            } else if is_push {
                if let UniformData::PushConstant(data) = &root_uniform.data {
                    if let Some((inner, _span)) = data.first() {
                        compute_offset(inner, &parts, key)?
                    } else {
                        return Err(ShaderError::InvalidKey(key.into()));
                    }
                } else {
                    return Err(ShaderError::InvalidKey(key.into()));
                }
            } else {
                compute_offset(&root_uniform.data, &parts, key)?
            };

            // Size/type check: the provided value must match leaf size
            let val_bytes = value.to_bytes();
            if val_bytes.len() != leaf_sz as usize {
                return Err(ShaderError::TypeMismatch(key.into()));
            }

            if is_storage {
                if let Some(blob) = self.storage_blobs.get_mut(&root) {
                    let start = rel_ofs as usize;
                    let end = start.saturating_add(val_bytes.len());
                    if end <= blob.len() {
                        blob[start..end].copy_from_slice(&val_bytes);
                        // Mark dirty for nested update
                        self.storage_dirty.insert(root.clone());
                        return Ok(());
                    }
                }
                Err(ShaderError::InvalidKey(key.into()))
            } else if is_push {
                if let Some(blob) = self.push_blobs.get_mut(&root) {
                    let start = rel_ofs as usize;
                    let end = start.saturating_add(val_bytes.len());
                    if end <= blob.len() {
                        blob[start..end].copy_from_slice(&val_bytes);
                        return Ok(());
                    }
                }
                Err(ShaderError::InvalidKey(key.into()))
            } else {
                // Classic uniform buffer bytes: base offset + relative offset
                let start = root_offset.saturating_add(rel_ofs) as usize;
                let end = start.saturating_add(val_bytes.len());
                if end <= self.uniform_bytes.len() {
                    self.uniform_bytes[start..end].copy_from_slice(&val_bytes);
                    return Ok(());
                }
                Err(ShaderError::InvalidKey(key.into()))
            }
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
        // Fast path: exact key was indexed
        if let Some((offset, size, uniform)) = self.uniforms.get(key) {
            if matches!(uniform.data, UniformData::PushConstant(_))
                && let Some(blob) = self.push_blobs.get(key)
            {
                let end = (*size as usize).min(blob.len());
                return Some(&blob[0..end]);
            }

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
            if let Some(blob) = self.push_blobs.get(root) {
                let start = *offset as usize;
                let end = start + *size as usize;
                return Some(&blob[start..end]);
            }

            // Fallback to uniform_bytes for classic uniforms
            let start = *offset as usize;
            let end = start + *size as usize;
            return Some(&self.uniform_bytes[start..end]);
        }

        // Slow path: compute offsets from array/struct shapes and return a slice
        if let Ok((root, parts)) = parse_key(key)
            && let Some((root_offset, _root_size, root_uniform)) = self.uniforms.get(&root)
        {
            let is_storage = matches!(root_uniform.data, UniformData::Storage(_));
            let is_push = matches!(root_uniform.data, UniformData::PushConstant(_));
            let (rel_ofs, leaf_sz) = if is_storage {
                if let UniformData::Storage(data) = &root_uniform.data {
                    if let Some((inner, _span, _)) = data.first() {
                        compute_offset(inner, &parts, key).ok()?
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else if is_push {
                if let UniformData::PushConstant(data) = &root_uniform.data {
                    if let Some((inner, _span)) = data.first() {
                        compute_offset(inner, &parts, key).ok()?
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                compute_offset(&root_uniform.data, &parts, key).ok()?
            };

            if is_storage {
                if let Some(blob) = self.storage_blobs.get(&root) {
                    let start = rel_ofs as usize;
                    let end = start + leaf_sz as usize;
                    if end <= blob.len() {
                        return Some(&blob[start..end]);
                    }
                }
            } else if is_push {
                if let Some(blob) = self.push_blobs.get(&root) {
                    let start = rel_ofs as usize;
                    let end = start + leaf_sz as usize;
                    if end <= blob.len() {
                        return Some(&blob[start..end]);
                    }
                }
            } else {
                let start = root_offset.saturating_add(rel_ofs) as usize;
                let end = start + leaf_sz as usize;
                if end <= self.uniform_bytes.len() {
                    return Some(&self.uniform_bytes[start..end]);
                }
            }
        }

        None
    }

    // ----- Storage dirty helpers (crate-private) -----
    pub(crate) fn is_storage_dirty(&self, root: &str) -> bool {
        self.storage_dirty.contains(root)
    }
    pub(crate) fn clear_storage_dirty(&mut self, root: &str) {
        self.storage_dirty.remove(root);
    }
}

// -------------------------------
// Key parsing and offset compute
// -------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Segment {
    Field(String),
    Index(usize),
}

fn parse_key(key: &str) -> Result<(String, Vec<Segment>), ShaderError> {
    // Extract root name: run until '.' or '['
    let mut chars = key.chars().peekable();
    let mut root = String::new();
    while let Some(&c) = chars.peek() {
        if c == '.' || c == '[' {
            break;
        }
        root.push(c);
        chars.next();
    }
    if root.is_empty() {
        return Err(ShaderError::InvalidKey(key.into()));
    }

    let mut parts = Vec::new();
    while let Some(c) = chars.peek().cloned() {
        match c {
            '.' => {
                // Consume '.' and parse field name
                chars.next();
                let mut name = String::new();
                while let Some(&d) = chars.peek() {
                    if d == '.' || d == '[' {
                        break;
                    }
                    name.push(d);
                    chars.next();
                }
                if name.is_empty() {
                    return Err(ShaderError::InvalidKey(key.into()));
                }
                parts.push(Segment::Field(name));
            }
            '[' => {
                // Parse index: '[' digits ']'
                chars.next(); // consume '['
                let mut num = String::new();
                while let Some(&d) = chars.peek() {
                    if d == ']' {
                        break;
                    }
                    if !d.is_ascii_digit() {
                        return Err(ShaderError::InvalidKey(key.into()));
                    }
                    num.push(d);
                    chars.next();
                }
                if chars.peek() == Some(&']') {
                    chars.next();
                } else {
                    return Err(ShaderError::InvalidKey(key.into()));
                }
                if num.is_empty() {
                    return Err(ShaderError::InvalidKey(key.into()));
                }
                let idx: usize = num
                    .parse()
                    .map_err(|_| ShaderError::InvalidKey(key.into()))?;
                parts.push(Segment::Index(idx));
            }
            _ => return Err(ShaderError::InvalidKey(key.into())),
        }
    }

    Ok((root, parts))
}

fn compute_offset(
    shape: &UniformData,
    parts: &[Segment],
    key: &str,
) -> Result<(u32, u32), ShaderError> {
    let mut ofs: u32 = 0;
    let mut cur = shape;

    let mut i = 0usize;
    while i < parts.len() {
        match cur {
            UniformData::Struct((fields, _span)) => match &parts[i] {
                Segment::Field(name) => {
                    let mut found = None;
                    for (fo, fname, f) in fields.iter() {
                        if fname == name {
                            found = Some((*fo, f));
                            break;
                        }
                    }
                    if let Some((fo, f)) = found {
                        ofs = ofs.saturating_add(fo);
                        cur = f;
                        i += 1;
                    } else {
                        return Err(ShaderError::FieldNotFound(name.clone()));
                    }
                }
                Segment::Index(_) => return Err(ShaderError::InvalidKey(key.into())),
            },
            UniformData::Array(items) => {
                if let Some((elem, count, stride)) = items.first() {
                    match &parts[i] {
                        Segment::Index(idx) => {
                            let n = *count as usize;
                            if *idx >= n {
                                return Err(ShaderError::IndexOutOfBounds {
                                    key: key.into(),
                                    index: *idx,
                                    len: n,
                                });
                            }
                            ofs = ofs.saturating_add((*idx as u32) * *stride);
                            cur = elem;
                            i += 1;
                        }
                        Segment::Field(_) => return Err(ShaderError::InvalidKey(key.into())),
                    }
                } else {
                    return Err(ShaderError::InvalidKey(key.into()));
                }
            }
            // Leaf types: cannot consume more parts
            _ => {
                break;
            }
        }
    }

    if i != parts.len() {
        // we still have leftover navigation but hit a leaf
        return Err(ShaderError::InvalidKey(key.into()));
    }

    Ok((ofs, cur.size()))
}
