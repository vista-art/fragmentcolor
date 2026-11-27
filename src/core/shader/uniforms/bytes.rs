//! `UniformStorage`: the CPU bytes of every uniform,
//! storage and immediate root, and the keys that address them.

use std::collections::{BinaryHeap, HashMap};
use std::mem;
use std::ops::Range;
use std::sync::{Arc, Weak};

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
    // What changed in each storage root on the CPU, and how far each RenderContext's buffer has caught up.
    storage_changes: HashMap<String, StorageChanges>,
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
            storage_changes: HashMap::new(),
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
        let len = blob.len();
        mark_storage(&mut self.storage_changes, root, 0..len);
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
                    if mem::discriminant(value) != mem::discriminant(&uniform.data) {
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
                            let len = blob.len();
                            mark_storage(&mut self.storage_changes, root, 0..len);
                        }
                    }
                    _ => {
                        // Nested field: copy at offset
                        let start = *offset as usize;
                        let data = value.to_bytes();
                        let end = start.saturating_add(data.len());
                        if end <= blob.len() {
                            blob[start..end].copy_from_slice(&data);
                            mark_storage(&mut self.storage_changes, root, start..end);
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
            if mem::discriminant(value) != mem::discriminant(leaf)
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
                mark_storage(&mut self.storage_changes, &root, start..end);
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

    /// The word ranges of the storage root `root` that the RenderContext `context` lacks.
    ///
    /// A context this shader uploaded to lacks the bytes set since that upload.
    /// One it never uploaded to, whose buffer another shader created, lacks each
    /// byte this shader ever set, and only those, so what the other shader keeps
    /// there stays. Ranges come sorted and apart, each starting and ending on a word,
    /// so the last one can reach past a root whose size isn't a multiple of 4, into its padding.
    pub(crate) fn list_changes(&self, root: &str, context: u64) -> Vec<Range<usize>> {
        self.storage_changes
            .get(root)
            .map_or_else(Vec::new, |changes| changes.list(context))
    }

    /// Records that `context` holds every change to the storage root `root`
    /// after an upload or the creation of its buffer. `token` lives as long as
    /// the context, so the changes forget a context that is gone at the next upload.
    pub(crate) fn record_upload(&mut self, root: &str, context: u64, token: &Arc<()>) {
        self.storage_changes
            .entry(root.to_string())
            .or_default()
            .record(context, token);
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

/// The changes to one storage root, and how far each RenderContext's buffer has caught up.
///
/// Each set takes the current epoch, and each upload ends one. A context holds
/// every change up to the epoch of its last upload, so it takes only the words
/// set after that, and never writes its CPU copy over what a compute pass wrote.
#[derive(Debug, Clone, Default)]
struct StorageChanges {
    /// Each set, as whole 4-byte words and the epoch it came in, in epoch order.
    log: Vec<Change>,
    /// How many entries the last compaction left, so the log compacts again once it doubles.
    compacted: usize,
    /// The epoch of the sets made now.
    epoch: u64,
    /// The oldest epoch a live context holds. Compaction raises older entries to it.
    floor: u64,
    /// How far each live RenderContext's buffer has caught up, by context id.
    contexts: HashMap<u64, Seen>,
}

/// One set of a storage root: the words it changed, and the epoch it came in.
#[derive(Debug, Clone)]
struct Change {
    range: Range<usize>,
    epoch: u64,
}

/// How far one RenderContext's buffer has caught up.
#[derive(Debug, Clone)]
struct Seen {
    /// The buffer holds every change up to this epoch.
    epoch: u64,
    /// Dies with the context, so its entry can go.
    token: Weak<()>,
}

impl StorageChanges {
    /// Records that the words in `range` changed.
    /// The log compacts once it doubles, so sets with no upload between them keep it short.
    fn mark(&mut self, range: Range<usize>) {
        let epoch = self.epoch;
        self.log.push(Change { range, epoch });
        if self.log.len() > (self.compacted * 2).max(64) {
            self.log = compact(mem::take(&mut self.log), self.floor);
            self.compacted = self.log.len();
        }
    }

    /// The ranges `context` lacks, sorted and apart:
    /// the ones set after its last upload, or every one ever set for a context that never uploaded.
    fn list(&self, context: u64) -> Vec<Range<usize>> {
        let first = self.contexts.get(&context).map_or(0, |seen| {
            self.log
                .partition_point(|change| change.epoch <= seen.epoch)
        });
        let ranges = self
            .log
            .iter()
            .skip(first)
            .map(|change| change.range.clone());
        merge_ranges(ranges.collect())
    }

    /// Records that `context` holds every change so far, and starts a new epoch. It forgets
    /// the contexts that are gone, and raises the floor to the oldest epoch a live context holds.
    fn record(&mut self, context: u64, token: &Arc<()>) {
        let token = Arc::downgrade(token);
        let epoch = self.epoch;
        self.contexts.insert(context, Seen { epoch, token });
        self.epoch += 1;
        self.contexts
            .retain(|_, seen| seen.token.strong_count() > 0);
        let oldest = self.contexts.values().map(|seen| seen.epoch).min();
        self.floor = oldest.unwrap_or(self.floor);
    }
}

/// Records in `changes` that the bytes in `range` of the storage root `root` changed on the CPU.
/// The range widens to whole 4-byte words, which is what a buffer copy takes.
fn mark_storage(changes: &mut HashMap<String, StorageChanges>, root: &str, range: Range<usize>) {
    let start = range.start & !3;
    let end = range.end.saturating_add(3) & !3;
    if start >= end {
        return;
    }
    changes
        .entry(root.to_string())
        .or_default()
        .mark(start..end);
}

/// Rewrites `log` as one entry per run of words that share the epoch of their last set.
///
/// Each word keeps the newest epoch that set it, so every context lacks the same
/// words in the result as in `log`. An epoch below `floor` rises to it, since every
/// live context holds it, so the runs they all hold join. The result is in epoch order.
fn compact(mut log: Vec<Change>, floor: u64) -> Vec<Change> {
    let mut points: Vec<usize> = log
        .iter()
        .flat_map(|change| [change.range.start, change.range.end])
        .collect();
    points.sort_unstable();
    points.dedup();
    log.sort_unstable_by_key(|change| change.range.start);
    let mut waiting = log.into_iter().peekable();
    let mut open = BinaryHeap::new();
    let mut runs: Vec<Change> = Vec::new();
    for pair in points.windows(2) {
        let &[start, end] = pair else {
            continue;
        };
        while let Some(change) = waiting.next_if(|change| change.range.start <= start) {
            open.push((change.epoch.max(floor), change.range.end));
        }
        while open.peek().is_some_and(|&(_, until)| until <= start) {
            open.pop();
        }
        let Some(&(epoch, _)) = open.peek() else {
            continue;
        };
        match runs.last_mut() {
            Some(last) if last.range.end == start && last.epoch == epoch => last.range.end = end,
            _ => runs.push(Change {
                range: start..end,
                epoch,
            }),
        }
    }
    runs.sort_by_key(|change| change.epoch);
    runs
}

/// Sorts `ranges` and joins the ones that overlap or touch, so they come sorted and apart.
fn merge_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_unstable_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for range in ranges {
        match merged.last_mut() {
            Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
            _ => merged.push(range),
        }
    }
    merged
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

#[cfg(test)]
// The expected uploads hold one byte range, such as `[0..4]`, which this lint reads as a typo.
#[allow(clippy::single_range_in_vec_init)]
mod tests {
    use super::*;

    /// The ranges a new context uploads after marking each `(start, end)` in order, as `(start, end)` pairs.
    fn marked(ranges: &[(usize, usize)]) -> Vec<(usize, usize)> {
        let mut storage = UniformStorage::new(&HashMap::new()).expect("empty storage");
        for &(start, end) in ranges {
            mark(&mut storage, start..end);
        }
        storage
            .list_changes("root", 1)
            .into_iter()
            .map(|range| (range.start, range.end))
            .collect()
    }

    /// Marks the words in `range` of the storage root `root` as changed.
    fn mark(storage: &mut UniformStorage, range: Range<usize>) {
        mark_storage(&mut storage.storage_changes, "root", range);
    }

    // Story: Each change widens to whole 4-byte words, as write_buffer takes them.
    #[test]
    fn changes_widen_to_whole_words() {
        assert_eq!(
            marked(&[(2, 4)]),
            [(0, 4)],
            "a 2-byte f16 in the second half"
        );
        assert_eq!(marked(&[(5, 10)]), [(4, 12)], "bytes that cross a word");
        assert_eq!(marked(&[(8, 8)]), [], "nothing");
    }

    // Story: Changes that overlap or touch join, and the others stay apart and sorted.
    #[test]
    fn changes_join_when_they_meet() {
        assert_eq!(marked(&[(16, 20), (0, 4)]), [(0, 4), (16, 20)], "apart");
        assert_eq!(marked(&[(0, 4), (4, 8), (8, 12)]), [(0, 12)], "in order");
        assert_eq!(marked(&[(0, 4), (8, 12), (4, 8)]), [(0, 12)], "gap filled");
        assert_eq!(
            marked(&[(0, 4), (8, 12), (16, 20), (6, 18)]),
            [(0, 20)],
            "one change over several"
        );
        assert_eq!(marked(&[(4, 12), (6, 8)]), [(4, 12)], "inside another");
    }
    // Story: Sets in descending order, with no render between them, join at upload
    // and keep the log short, so no set pays for the ones before it.
    #[test]
    fn many_changes_merge_once_the_log_doubles() {
        // Arrange
        let mut storage = UniformStorage::new(&HashMap::new()).expect("empty storage");

        // Act
        for word in (0..1000).rev() {
            mark(&mut storage, word * 4..word * 4 + 4);
        }

        // Assert
        let kept = storage.storage_changes["root"].log.len();
        assert!(kept <= 128, "{kept} entries kept");
        assert_eq!(
            storage.list_changes("root", 1),
            [0..4000],
            "one range covers every word"
        );
    }

    // Story: Each context catches up on its own. It takes the ranges set since its last
    // upload, the ones another context took included, and never one it already holds.
    #[test]
    fn each_context_catches_up_on_its_own() {
        // Arrange
        let mut storage = UniformStorage::new(&HashMap::new()).expect("empty storage");
        let (x, y) = (1, 2);
        let (x_token, y_token) = (Arc::new(()), Arc::new(()));
        storage.record_upload("root", x, &x_token);
        storage.record_upload("root", y, &y_token);

        // Act
        mark(&mut storage, 0..4);
        let x_first = storage.list_changes("root", x);
        storage.record_upload("root", x, &x_token);
        mark(&mut storage, 8..12);
        let y_after = storage.list_changes("root", y);
        storage.record_upload("root", y, &y_token);
        let x_after = storage.list_changes("root", x);
        storage.record_upload("root", x, &x_token);
        let x_again = storage.list_changes("root", x);

        // Assert
        assert_eq!(x_first, [0..4], "x takes its range");
        assert_eq!(
            y_after,
            [0..4, 8..12],
            "y takes both, with 0..4, which x took"
        );
        assert_eq!(x_after, [8..12], "x takes only 8..12, which y took");
        assert!(x_again.is_empty(), "x holds every change");
    }

    // Story: Another shader created the buffer a context binds, so this shader never
    // uploaded to it. The context takes every range this shader ever set, and no other.
    #[test]
    fn a_new_context_takes_every_range_ever_set() {
        // Arrange
        let mut storage = UniformStorage::new(&HashMap::new()).expect("empty storage");
        let (x, z) = (1, 3);
        let x_token = Arc::new(());
        mark(&mut storage, 0..4);
        storage.record_upload("root", x, &x_token);
        mark(&mut storage, 8..12);
        storage.record_upload("root", x, &x_token);
        mark(&mut storage, 16..20);

        // Act
        let x_lacks = storage.list_changes("root", x);
        let z_lacks = storage.list_changes("root", z);

        // Assert
        assert_eq!(x_lacks, [16..20], "x took the others");
        assert_eq!(z_lacks, [0..4, 8..12, 16..20], "z took none");
    }

    // Story: One context uploads each frame and another lags behind. Once the log
    // compacts, each still takes exactly what it lacks, and the log stays short.
    #[test]
    fn compaction_keeps_what_each_context_lacks() {
        // Arrange
        let mut storage = UniformStorage::new(&HashMap::new()).expect("empty storage");
        let (x, y) = (1, 2);
        let (x_token, y_token) = (Arc::new(()), Arc::new(()));
        storage.record_upload("root", x, &x_token);
        storage.record_upload("root", y, &y_token);

        // Act
        for frame in 0..1000 {
            let start = (frame % 10) * 8;
            mark(&mut storage, start..start + 4);
            storage.record_upload("root", x, &x_token);
        }
        mark(&mut storage, 200..204);

        // Assert
        let places = (0..10).map(|place| place * 8..place * 8 + 4);
        let missed: Vec<Range<usize>> = places.chain([200..204]).collect();
        let kept = storage.storage_changes["root"].log.len();
        assert_eq!(
            storage.list_changes("root", x),
            [200..204],
            "x took the rest"
        );
        assert_eq!(storage.list_changes("root", y), missed, "y took none");
        assert!(kept <= 128, "{kept} entries kept");
    }

    // Story: Every context keeps up while the app sets touching words in turn, so
    // compaction joins the runs they all hold and the log stays one entry long.
    #[test]
    fn runs_every_context_holds_join() {
        // Arrange
        let mut storage = UniformStorage::new(&HashMap::new()).expect("empty storage");
        let x = 1;
        let x_token = Arc::new(());

        // Act
        for word in 0..1000 {
            mark(&mut storage, word * 4..word * 4 + 4);
            storage.record_upload("root", x, &x_token);
        }

        // Assert
        let changes = &storage.storage_changes["root"];
        assert!(
            changes.log.len() <= 64,
            "{} entries kept",
            changes.log.len()
        );
        assert!(
            storage.list_changes("root", x).is_empty(),
            "x holds them all"
        );
        assert_eq!(
            storage.list_changes("root", 2),
            [0..4000],
            "a new context takes every word"
        );
    }

    // Story: A Renderer that is gone leaves its context's entry until the next upload,
    // which forgets it, so the floor follows the contexts still alive.
    #[test]
    fn a_context_that_is_gone_is_forgotten() {
        // Arrange
        let mut storage = UniformStorage::new(&HashMap::new()).expect("empty storage");
        let (x, y) = (1, 2);
        let (x_token, y_token) = (Arc::new(()), Arc::new(()));
        storage.record_upload("root", x, &x_token);
        storage.record_upload("root", y, &y_token);

        // Act
        drop(y_token);
        mark(&mut storage, 0..4);
        storage.record_upload("root", x, &x_token);

        // Assert
        let changes = &storage.storage_changes["root"];
        let known: Vec<u64> = changes.contexts.keys().copied().collect();
        assert_eq!(known, [x], "only x is left");
        assert_eq!(
            changes.floor, changes.contexts[&x].epoch,
            "the floor follows x alone"
        );
    }
}
