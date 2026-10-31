//! `UniformStorage`: the CPU bytes of every uniform,
//! storage and immediate root, and the keys that address them.

use std::collections::{HashMap, HashSet};

use crate::ShaderError;

use super::input::refuse_count;

use super::{ArrayElement, PushEntry, StorageEntry, StructField, StructShape};
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
    pub(crate) fn new(uniforms: &HashMap<String, Uniform>) -> Result<Self, ShaderError> {
        let mut storage = Self {
            uniform_bytes: Vec::new(),
            uniforms: HashMap::new(),
            storage_blobs: HashMap::new(),
            storage_dirty: HashSet::new(),
            push_blobs: HashMap::new(),
        };

        storage.extend(uniforms)?;

        Ok(storage)
    }

    /// Extend the block with a HashMap of key->UniformData
    fn extend(&mut self, uniforms: &HashMap<String, Uniform>) -> Result<(), ShaderError> {
        for uniform in uniforms.values() {
            self.add_uniform(uniform)?;
        }
        Ok(())
    }

    /// Add a single uniform to the storage.
    ///
    /// Uniforms are cached by name. A Struct uniform will be flattened as bytes.
    ///
    /// We will index both the uniform name and its fields with the dot notation.
    /// For example, for a struct uniform named `light` with fields `position` and `color`,
    /// we will index the uniform as `light` and the fields as `light.position` and `light.color`.
    /// Array elements are not indexed. A key like `u.items[2].v` resolves through `locate_key`.
    pub(crate) fn add_uniform(&mut self, uniform: &Uniform) -> Result<(), ShaderError> {
        // For storage buffers AND push constants, we keep a separate CPU blob
        // and do not write to uniform_bytes.
        // For other value types, append to uniform_bytes and index by offset.
        let base_offset = self.uniform_bytes.len() as u32;

        let is_storage = matches!(uniform.data, UniformData::Storage(_));
        let is_push = matches!(uniform.data, UniformData::PushConstant(_));
        if !is_storage && !is_push {
            // Reflected values are all zero, so the root starts as zero bytes.
            let size = uniform.data.size() as usize;
            reserve(&mut self.uniform_bytes, &uniform.name, size)?;
            self.uniform_bytes.resize(base_offset as usize + size, 0);
        }

        // Determine span for indexing; push constants report 0 via size(),
        // so compute explicitly.
        let mut index_size = uniform.data.size();
        if let UniformData::PushConstant(data) = &uniform.data
            && let Some(PushEntry { span, .. }) = data.first()
        {
            index_size = *span;
        }

        self.uniforms.insert(
            uniform.name.clone(),
            (base_offset, index_size, uniform.clone()),
        );

        // If the Uniform is a struct, we also index its fields to allow granular access
        if let UniformData::Struct(StructShape { fields, .. }) = &uniform.data {
            for StructField {
                offset: field_offset,
                name: field_name,
                ty: field,
            } in fields
            {
                let Some(field_data) = field.first() else {
                    continue;
                };
                let key = format!("{}.{}", uniform.name, field_name);
                self.uniforms.insert(
                    key.clone(),
                    (
                        base_offset + field_offset,
                        field_data.size(),
                        Uniform {
                            name: key.clone(),
                            group: uniform.group,
                            binding: uniform.binding,
                            stages: uniform.stages,
                            data: field_data.clone(),
                        },
                    ),
                );
            }
        }

        // Storage buffers have their own GPU buffer; create a CPU blob and index fields with offsets
        if let UniformData::Storage(data) = &uniform.data
            && let Some(StorageEntry { inner, span, .. }) = data.iter().next()
        {
            self.storage_blobs
                .insert(uniform.name.clone(), allocate_zeros(&uniform.name, *span)?);

            // Index fields with their declared offsets relative to the storage blob
            if let Some(inner_shape) = inner.first()
                && let UniformData::Struct(StructShape { fields, .. }) = inner_shape
            {
                for StructField {
                    offset: field_offset,
                    name: field_name,
                    ty: field,
                } in fields
                {
                    let Some(field_data) = field.first() else {
                        continue;
                    };
                    let key = format!("{}.{}", uniform.name, field_name);
                    self.uniforms.insert(
                        key.clone(),
                        (
                            *field_offset,
                            field_data.size(),
                            Uniform {
                                name: key,
                                group: uniform.group,
                                binding: uniform.binding,
                                stages: uniform.stages,
                                data: field_data.clone(),
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
                        stages: uniform.stages,
                        data: uniform.data.clone(),
                    },
                ),
            );
        }

        // Push constants: maintain a per-root CPU blob similar to storage
        if let UniformData::PushConstant(data) = &uniform.data
            && let Some(PushEntry { inner, span }) = data.iter().next()
        {
            self.push_blobs
                .insert(uniform.name.clone(), allocate_zeros(&uniform.name, *span)?);

            // Index fields for direct access if it's a struct
            if let Some(inner_shape) = inner.first()
                && let UniformData::Struct(StructShape { fields, .. }) = inner_shape
            {
                for StructField {
                    offset: field_offset,
                    name: field_name,
                    ty: field,
                } in fields
                {
                    let Some(field_data) = field.first() else {
                        continue;
                    };
                    let key = format!("{}.{}", uniform.name, field_name);
                    self.uniforms.insert(
                        key.clone(),
                        (
                            *field_offset,
                            field_data.size(),
                            Uniform {
                                name: key,
                                group: uniform.group,
                                binding: uniform.binding,
                                stages: uniform.stages,
                                data: field_data.clone(),
                            },
                        ),
                    );
                }
            }
        }
        Ok(())
    }

    /// Replaces the bytes of the storage root `root`, which must be exactly its size.
    pub fn set_storage_bytes(&mut self, root: &str, bytes: &[u8]) -> Result<(), ShaderError> {
        let blob = self
            .storage_blobs
            .get_mut(root)
            .ok_or_else(|| ShaderError::UniformNotFound(root.into()))?;
        replace_blob(root, blob, bytes)?;
        self.storage_dirty.insert(root.to_string());
        Ok(())
    }

    /// Replaces the bytes of the push-constant root `root`, which must be exactly its size.
    pub fn set_push_bytes(&mut self, root: &str, bytes: &[u8]) -> Result<(), ShaderError> {
        let blob = self
            .push_blobs
            .get_mut(root)
            .ok_or_else(|| ShaderError::UniformNotFound(root.into()))?;
        replace_blob(root, blob, bytes)
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

            // A texture takes the incoming id and keeps the slot's reflected metadata.
            // Id 0 unsets the slot.
            match (&uniform.data, value) {
                (UniformData::Texture(existing), UniformData::Texture(incoming)) => {
                    let merged = crate::texture::TextureMeta {
                        id: incoming.id,
                        ..existing.clone()
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
                        if let Some(StorageEntry { inner, span, .. }) = data.iter().next()
                            && let Some(shape) = inner.first()
                        {
                            // Write full storage blob from inner
                            let data = shape.to_bytes();
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
                        if let Some(PushEntry { inner, span }) = data.iter().next()
                            && let Some(shape) = inner.first()
                        {
                            let data = shape.to_bytes();
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
            // Slow path: keys with array indices like "data.arr[3].field" or "u[2]"
            let (root, blob, start, leaf) = self.locate_key(key)?;
            let bytes = value.to_bytes();
            if std::mem::discriminant(value) != std::mem::discriminant(leaf)
                || bytes.len() != leaf.size() as usize
            {
                return Err(ShaderError::TypeMismatch(key.into()));
            }

            let target = match blob {
                Blob::Uniform => Some(&mut self.uniform_bytes),
                Blob::Storage => self.storage_blobs.get_mut(&root),
                Blob::Push => self.push_blobs.get_mut(&root),
            };
            let end = start.saturating_add(bytes.len());
            match target {
                Some(target) if end <= target.len() => target[start..end].copy_from_slice(&bytes),
                _ => return Err(ShaderError::InvalidKey(key.into())),
            }
            if blob == Blob::Storage {
                self.storage_dirty.insert(root);
            }
            Ok(())
        }
    }

    /// The declared type of `key`, including nested and indexed keys like `buf.items[2].v`.
    pub(crate) fn shape(&self, key: &str) -> Result<&UniformData, ShaderError> {
        if let Some((_, _, uniform)) = self.uniforms.get(key) {
            return Ok(&uniform.data);
        }
        self.locate_key(key).map(|(_, _, _, leaf)| leaf)
    }

    /// Finds a key that the index does not hold by walking its root's declared shape.
    /// Returns the root name, the blob holding the root, the key's byte offset in
    /// that blob, and the key's declared type.
    fn locate_key(&self, key: &str) -> Result<(String, Blob, usize, &UniformData), ShaderError> {
        let (root, parts) = parse_key(key)?;
        let (root_offset, _, uniform) = self
            .uniforms
            .get(&root)
            .ok_or_else(|| ShaderError::UniformNotFound(root.clone()))?;
        let (blob, base, shape) = match &uniform.data {
            UniformData::Storage(data) => (
                Blob::Storage,
                0,
                data.first()
                    .and_then(|StorageEntry { inner, .. }| inner.first()),
            ),
            UniformData::PushConstant(data) => (
                Blob::Push,
                0,
                data.first()
                    .and_then(|PushEntry { inner, .. }| inner.first()),
            ),
            other => (Blob::Uniform, *root_offset, Some(other)),
        };
        let shape = shape.ok_or_else(|| ShaderError::InvalidKey(key.into()))?;
        let (offset, leaf) = locate(shape, &parts, key)?;
        Ok((root, blob, base.saturating_add(offset) as usize, leaf))
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
        let (root, blob, start, leaf) = self.locate_key(key).ok()?;
        let end = start.saturating_add(leaf.size() as usize);
        let source = match blob {
            Blob::Uniform => &self.uniform_bytes,
            Blob::Storage => self.storage_blobs.get(&root)?,
            Blob::Push => self.push_blobs.get(&root)?,
        };
        source.get(start..end)
    }

    // ----- Storage dirty helpers (crate-private) -----
    pub(crate) fn is_storage_dirty(&self, root: &str) -> bool {
        self.storage_dirty.contains(root)
    }
    pub(crate) fn clear_storage_dirty(&mut self, root: &str) {
        self.storage_dirty.remove(root);
    }
    /// True iff the shader's source declared a top-level storage binding
    /// named `root` (i.e. there is a CPU-side blob for it).
    pub(crate) fn has_storage(&self, root: &str) -> bool {
        self.storage_blobs.contains_key(root)
    }
}

/// Reserves `size` more bytes in `bytes` for the root `name`, refusing a size the process can't hold.
fn reserve(bytes: &mut Vec<u8>, name: &str, size: usize) -> Result<(), ShaderError> {
    bytes.try_reserve_exact(size).map_err(|_| {
        ShaderError::TypeMismatch(format!(
            "{name}: {size} bytes is more memory than this process can hold"
        ))
    })
}

/// `span` zero bytes for the root `name`. A reservation probes for the memory first,
/// and `vec!` then asks the allocator for zeroed pages, which stay untouched until written.
fn allocate_zeros(name: &str, span: u32) -> Result<Vec<u8>, ShaderError> {
    reserve(&mut Vec::new(), name, span as usize)?;
    Ok(vec![0u8; span as usize])
}

/// Copies `bytes` over `blob`, refusing any other length.
fn replace_blob(root: &str, blob: &mut [u8], bytes: &[u8]) -> Result<(), ShaderError> {
    if bytes.len() != blob.len() {
        return Err(refuse_count(root, blob.len(), bytes.len()));
    }
    blob.copy_from_slice(bytes);
    Ok(())
}

// -------------------------------
// Key parsing and offset compute
// -------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Segment {
    Field(String),
    Index(usize),
}

/// The CPU-side buffer that holds a root's bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Blob {
    Uniform,
    Storage,
    Push,
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

/// Walks `parts` into `shape`. Returns the byte offset and the declared type that the path reaches.
fn locate<'a>(
    shape: &'a UniformData,
    parts: &[Segment],
    key: &str,
) -> Result<(u32, &'a UniformData), ShaderError> {
    let mut ofs: u32 = 0;
    let mut cur = shape;

    let mut i = 0usize;
    while i < parts.len() {
        match cur {
            UniformData::Struct(StructShape { fields, .. }) => match &parts[i] {
                Segment::Field(name) => {
                    let mut found = None;
                    for StructField {
                        offset: fo,
                        name: fname,
                        ty: f,
                    } in fields.iter()
                    {
                        if fname == name {
                            if let Some(field_shape) = f.first() {
                                found = Some((*fo, field_shape));
                            }
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
                if let Some(ArrayElement { ty, count, stride }) = items.first()
                    && let Some(elem_shape) = ty.first()
                {
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
                            cur = elem_shape;
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

    Ok((ofs, cur))
}
