use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg(python)]
use pyo3::prelude::*;

pub mod error;
pub use error::*;

pub mod vertex;
pub use vertex::*;
use vertex::{POSITION, explain_position};

mod data;
pub use data::VertexData;
pub(crate) use data::{Shape, describe};

pub mod primitives;
pub use primitives::*;

pub(crate) mod builtins;

mod platform;

use crate::renderer::{RenderContext, RendererError, VertexLayouts, check_limit};
use crate::{PassObject, Renderable, Shader, ShaderError};

#[derive(Clone, Debug)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[lsp_doc("docs/api/geometry/mesh/mesh.md")]
pub struct Mesh {
    pub(crate) object: Arc<MeshObject>,
    pub(crate) pass: Arc<crate::pass::PassObject>,
}

crate::impl_fc_kind!(Mesh, "Mesh");

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh {
    #[lsp_doc("docs/api/geometry/mesh/new.md")]
    pub fn new() -> Self {
        Self {
            object: Arc::new(MeshObject::new()),
            pass: Arc::new(crate::pass::PassObject::new(
                "Mesh Internal Pass",
                crate::pass::PassType::Render,
            )),
        }
    }

    #[lsp_doc("docs/api/geometry/mesh/from_vertices.md")]
    pub fn from_vertices<I, V>(verts: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<Vertex>,
    {
        let mesh = Mesh::new();
        mesh.add_vertices(verts);
        mesh
    }

    #[lsp_doc("docs/api/geometry/mesh/add_vertex.md")]
    pub fn add_vertex<V: Into<Vertex>>(&self, v: V) {
        self.object.add_vertex(v.into());
    }

    #[lsp_doc("docs/api/geometry/mesh/add_vertices.md")]
    pub fn add_vertices<I, V>(&self, vertices: I)
    where
        I: IntoIterator<Item = V>,
        V: Into<Vertex>,
    {
        for vertex in vertices {
            self.object.add_vertex(vertex.into());
        }
    }

    #[lsp_doc("docs/api/geometry/mesh/add_instance.md")]
    pub fn add_instance<T: Into<Instance>>(&self, instance_buffer: T) {
        self.object.add_instance(instance_buffer.into());
    }

    #[lsp_doc("docs/api/geometry/mesh/add_instances.md")]
    pub fn add_instances<I, T>(&self, instances: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<Instance>,
    {
        for instance in instances {
            self.object.add_instance(instance.into());
        }
    }

    #[lsp_doc("docs/api/geometry/mesh/clear_instances.md")]
    pub fn clear_instances(&self) {
        self.object.clear_instances();
    }

    /// Override how many instances to draw (when not using per-instance attributes).
    /// Cleared automatically by `add_instance` / `add_instances` / `clear_instances`.
    #[lsp_doc("docs/api/geometry/mesh/set_instance_count.md")]
    pub fn set_instance_count(&self, n: u32) {
        *self.object.override_instances.write() = Some(n);
        self.object.invalidate_cache();
    }

    #[lsp_doc("docs/api/geometry/mesh/set_indices.md")]
    pub fn set_indices<I: IntoIterator<Item = u32>>(&self, indices: I) {
        self.object.set_indices(indices.into_iter().collect());
    }

    #[lsp_doc("docs/api/geometry/mesh/clear_indices.md")]
    pub fn clear_indices(&self) {
        self.object.clear_indices();
    }
}

// -----------------------------
// Renderable impl ( for Mesh quick-view)
// -----------------------------
/// The quick view draws the mesh with a shader built from its first vertex.
/// That shader records no reads, so the mesh's other shaders keep their own rules.
/// It checks the mesh itself whenever it changed, on every render that draws it,
/// a `Pass::require` of the mesh included, and a mesh it refuses makes `render` say why.
impl Renderable for Mesh {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        if self.object.verts.read().is_empty() {
            return vec![self.pass.clone()].into();
        }
        let shader = self.pass.ensure_shader(|| {
            // Copied once, so no vertex guard is held while the shader compiles or checks.
            let first = self.object.verts.read().first().cloned();
            match first.as_ref().map(Shader::from_vertex) {
                Some(Ok(shader)) => shader.object,
                _ => Shader::default().object,
            }
        });
        shader.preview(&self.object);
        vec![self.pass.clone()].into()
    }

    /// The quick view's pass, without attaching the mesh to a shader of its own.
    fn peek_passes(&self) -> Arc<[Arc<PassObject>]> {
        vec![self.pass.clone()].into()
    }
}

// -----------------------------
// Internals
// -----------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct VertexKey {
    position: PosBits,
    properties: Vec<(String, PropBits)>, // sorted by key
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PosBits {
    P2([u32; 2]),
    P3([u32; 3]),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PropBits {
    B(Vec<u8>),
} // raw bytes

impl From<&Vertex> for VertexKey {
    fn from(v: &Vertex) -> Self {
        let pos = match v.dimensions {
            0..=2 => PosBits::P2([v.position.0.x.to_bits(), v.position.0.y.to_bits()]),
            _ => PosBits::P3([
                v.position.0.x.to_bits(),
                v.position.0.y.to_bits(),
                v.position.0.z.to_bits(),
            ]),
        };
        let mut props: Vec<(String, PropBits)> = v
            .properties
            .iter()
            .map(|(k, val)| (k.clone(), PropBits::B(val.to_key())))
            .collect();
        props.sort_by(|a, b| a.0.cmp(&b.0));
        VertexKey {
            position: pos,
            properties: props,
        }
    }
}

#[derive(Debug)]
pub(crate) struct MeshObject {
    // CPU side storage
    pub(crate) verts: RwLock<Vec<Vertex>>, // original order
    pub(crate) insts: RwLock<Vec<Instance>>,

    /// What the last pack made. Only `ensure_packed` writes it.
    packed: RwLock<Packed>,
    /// Count the changes to each stream. A stream whose count moved past the one
    /// its packed bytes hold packs again. A mutator bumps them after it changes the data
    /// and before `generation`, both before it lets the data go, so a pack that reads
    /// a count sees that change, and a pack that sees a change finds `generation` moved.
    vertex_changes: AtomicU64,
    instance_changes: AtomicU64,

    pub(crate) indices: RwLock<Vec<u32>>, // the list set_indices gave
    pub(crate) indices_overridden: RwLock<bool>, // user supplied indices via set_indices

    // Schemas
    pub(crate) vertex_schema: RwLock<Option<VertexSchema>>, // derived from first vertex
    pub(crate) instance_schema: RwLock<Option<VertexSchema>>, // derived from first instance

    /// The format the first shader to read each untyped attribute gave it.
    /// A leaf lock: never take another lock while holding this one.
    pub(crate) types: RwLock<AttributeTypes>,
    /// What each attached shader reads and the layouts it drew the mesh with, by
    /// `ShaderObject::id`. Every vertex and instance holds what any of them reads,
    /// and a shader's entry goes when it no longer holds the mesh. Taken after the
    /// shader's mesh list, which an attach holds from `declare` to the push.
    required: RwLock<HashMap<u64, Attached>>,

    // Optional override for instance count (allows drawing without instance buffer)
    override_instances: RwLock<Option<u32>>,

    // Axis-aligned bounding box of the mesh's vertex positions, in local
    // (mesh) space. Updated incrementally as `add_vertex` runs. The
    // renderer reads this through `MeshObject::aabb_local` to compute a
    // world-space centroid for translucent eye-Z sort. Sentinel pair
    // (`+inf` / `-inf`) flags a degenerate (empty) mesh.
    aabb_min: RwLock<glam::Vec3>,
    aabb_max: RwLock<glam::Vec3>,

    /// GPU buffers on the device that drew the mesh last, created lazily.
    /// An upload holds this lock from the pack to the cache it fills.
    gpu: RwLock<Option<GpuStreams>>,

    /// The last upload, with the context it belongs to.
    cache: RwLock<Option<Cached>>,
    /// Counts the changes to the mesh. A cached result or layout from an older one is stale.
    generation: AtomicU64,
}

/// The bytes and indices the last pack made, with the change counts they hold.
#[derive(Debug, Default)]
struct Packed {
    /// The unique vertices in the mesh layout, or every vertex in order with `set_indices`.
    verts: Vec<u8>,
    insts: Vec<u8>,
    /// The checked list the mesh draws: derived from the vertices, or the one `set_indices` gave.
    indices: Vec<u32>,
    /// The instances `insts` holds, those without properties included.
    rows: usize,
    /// The layouts `verts` and `insts` follow.
    schemas: Arc<Schemas>,
    /// The `vertex_changes` and `instance_changes` the bytes come from, `None` until packed.
    vertices_at: Option<u64>,
    instances_at: Option<u64>,
}

/// The vertex and instance layouts the bytes of one pack follow.
#[derive(Clone, Debug, Default)]
pub(crate) struct Schemas {
    pub(crate) vertex: Option<VertexSchema>,
    pub(crate) instance: Option<VertexSchema>,
}

impl Packed {
    /// Marks the streams `repacked` names unpacked, after an upload that failed to write them.
    fn forget(&mut self, repacked: Repacked) {
        if repacked.vertices {
            self.vertices_at = None;
        }
        if repacked.instances {
            self.instances_at = None;
        }
    }
}

/// The streams a pack packed again.
#[derive(Clone, Copy, Debug, Default)]
struct Repacked {
    vertices: bool,
    instances: bool,
}

/// What one attached shader reads from the mesh, and the
/// vertex layouts it read, with the generation they came from.
#[derive(Debug, Default)]
struct Attached {
    reads: AttributeTypes,
    layouts: Option<(u64, VertexLayouts)>,
}

/// An upload and the context whose device holds its buffers,
/// in one value, so an upload never reaches another context.
#[derive(Debug)]
struct Cached {
    owner: u64,
    upload: Upload,
}

/// The buffers an upload wrote, the counts that draw them and the layouts their
/// bytes follow, all from one pack, with the mesh's generation when the upload began.
#[derive(Clone, Debug)]
pub(crate) struct Upload {
    pub(crate) buffers: VertexBuffers,
    pub(crate) counts: DrawCounts,
    pub(crate) schemas: Arc<Schemas>,
    pub(crate) generation: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VertexSchema {
    pub(crate) stride: u64,
    pub(crate) fields: Vec<Field>,
}

impl VertexSchema {
    fn new(fields: Vec<Field>) -> Self {
        let stride = fields.iter().map(|field| field.size).sum();
        Self { stride, fields }
    }

    /// Whether this is the instance layout of a `Model`, without building one to compare.
    fn is_model(&self) -> bool {
        let columns = self
            .fields
            .iter()
            .map(|field| (field.name.as_str(), field.fmt));
        columns.eq(MODEL_COLUMNS.map(|name| (name, wgpu::VertexFormat::Float32x4)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) fmt: wgpu::VertexFormat,
    pub(crate) size: u64,
}

impl Field {
    fn new(name: &str, fmt: wgpu::VertexFormat) -> Self {
        Self {
            name: name.to_string(),
            fmt,
            size: fmt.size(),
        }
    }
}

/// The format a shader gave each untyped attribute, by stream and name.
#[derive(Clone, Debug, Default)]
pub(crate) struct AttributeTypes {
    pub(crate) vertex: HashMap<String, wgpu::VertexFormat>,
    pub(crate) instance: HashMap<String, wgpu::VertexFormat>,
}

impl AttributeTypes {
    fn is_empty(&self) -> bool {
        self.vertex.is_empty() && self.instance.is_empty()
    }
}

/// What a shader's attach records on a mesh: the types it gives
/// untyped attributes, and the attributes it reads, in the format it reads each.
/// `columns` is set when the check assumed the Model columns as the instance layout.
#[derive(Clone, Debug, Default)]
pub(crate) struct Binding {
    pub(crate) types: AttributeTypes,
    pub(crate) read: AttributeTypes,
    pub(crate) columns: bool,
}

impl Binding {
    /// The same binding with no reads, for a shader that checks the mesh itself before it draws.
    pub(crate) fn without_reads(self) -> Self {
        Self {
            read: AttributeTypes::default(),
            ..self
        }
    }
}

/// A mesh's buffers on the device of one `Renderer`. Another `Renderer`
/// that draws the mesh uploads buffers of its own in their place.
#[derive(Debug)]
struct GpuStreams {
    /// The `RenderContext::id` of the context whose device holds the buffers.
    owner: u64,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_capacity: u64,
}

impl MeshObject {
    fn new() -> Self {
        Self {
            verts: RwLock::new(Vec::new()),
            insts: RwLock::new(Vec::new()),
            packed: RwLock::new(Packed::default()),
            vertex_changes: AtomicU64::new(0),
            instance_changes: AtomicU64::new(0),
            indices: RwLock::new(Vec::new()),
            indices_overridden: RwLock::new(false),
            vertex_schema: RwLock::new(None),
            instance_schema: RwLock::new(None),
            types: RwLock::new(AttributeTypes::default()),
            required: RwLock::new(HashMap::new()),
            override_instances: RwLock::new(None),
            aabb_min: RwLock::new(glam::Vec3::splat(f32::INFINITY)),
            aabb_max: RwLock::new(glam::Vec3::splat(f32::NEG_INFINITY)),
            gpu: RwLock::new(None),
            cache: RwLock::new(None),
            generation: AtomicU64::new(0),
        }
    }

    fn add_vertex(&self, v: Vertex) {
        // Grow the local-space AABB before stashing the Vertex. 2D positions
        // contribute z = 0, which is the same value the vertex shader sees
        // (the `position` slot zero-pads the missing components).
        let p = glam::Vec3::new(v.position.0.x, v.position.0.y, v.position.0.z);
        let mut lo = self.aabb_min.write();
        let mut hi = self.aabb_max.write();
        *lo = lo.min(p);
        *hi = hi.max(p);
        drop(lo);
        drop(hi);
        let mut verts = self.verts.write();
        verts.push(v);
        self.touch_vertices();
        drop(verts);
        #[cfg(test)]
        crate::hooks::run("added");
    }

    /// Local-space AABB of every Vertex added so far. Returns the sentinel
    /// pair (`+inf`, `-inf`) for an empty mesh; callers detect this and
    /// fall back to a centroid of zero. Internal-only — glam types stay
    /// inside the crate per the no-glam-on-public-API rule.
    pub(crate) fn aabb_local(&self) -> (glam::Vec3, glam::Vec3) {
        (*self.aabb_min.read(), *self.aabb_max.read())
    }

    fn add_instance(&self, i: Instance) {
        let mut insts = self.insts.write();
        insts.push(i);
        // Adding a real instance invalidates any prior set_instance_count override
        // (otherwise the GPU would draw past the actual buffer end).
        *self.override_instances.write() = None;
        self.touch_instances();
    }

    fn clear_instances(&self) {
        let mut insts = self.insts.write();
        insts.clear();
        // The next instance sets the layout again, but the model columns stay.
        let mut schema = self.instance_schema.write();
        if schema.as_ref().is_some_and(|schema| !schema.is_model()) {
            *schema = None;
        }
        drop(schema);
        // No buffer and no count override. A shader that reads instances refuses the empty list.
        *self.override_instances.write() = None;
        self.touch_instances();
    }

    fn set_indices(&self, indices: Vec<u32>) {
        // Taken in the pack's order: whether the indices were given, then the list.
        let mut overridden = self.indices_overridden.write();
        *self.indices.write() = indices;
        *overridden = true;
        // Vertices need to be re-packed in insertion order (no dedup).
        self.touch_vertices();
    }

    fn clear_indices(&self) {
        let mut overridden = self.indices_overridden.write();
        *overridden = false;
        // The next pack derives the indices again, deduplicating the vertices.
        self.touch_vertices();
    }

    /// Marks the vertices changed, so the next upload packs and checks them again.
    fn touch_vertices(&self) {
        self.vertex_changes.fetch_add(1, Ordering::AcqRel);
        self.invalidate_cache();
    }

    /// Marks the instances changed, so the next upload packs and checks them again.
    fn touch_instances(&self) {
        self.instance_changes.fetch_add(1, Ordering::AcqRel);
        self.invalidate_cache();
    }

    /// Marks every cached result and layout stale. It takes no lock, so a
    /// mutator may call it whatever it holds.
    pub(crate) fn invalidate_cache(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }

    /// The count of changes to the mesh, which moves on every one.
    pub(crate) fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    /// The layouts `build` makes for the attached shader `shader` from the upload at
    /// `generation`, kept until the mesh changes or the shader lets the mesh go.
    /// A shader that doesn't hold the mesh keeps none.
    pub(crate) fn fetch_layouts(
        &self,
        shader: u64,
        generation: u64,
        build: impl FnOnce() -> Result<VertexLayouts, RendererError>,
    ) -> Result<VertexLayouts, RendererError> {
        if let Some(attached) = self.required.read().get(&shader)
            && let Some((kept, layouts)) = attached.layouts.as_ref()
            && *kept == generation
        {
            return Ok(layouts.clone());
        }
        let layouts = build()?;
        if let Some(attached) = self.required.write().get_mut(&shader) {
            attached.layouts = Some((generation, layouts.clone()));
        }
        Ok(layouts)
    }

    /// Whether only Models give this mesh instances:
    /// it declares the Model columns and holds no instance of its own.
    pub(crate) fn awaits_models(&self) -> bool {
        self.insts.read().is_empty()
            && self
                .instance_schema
                .read()
                .as_ref()
                .is_some_and(VertexSchema::is_model)
    }

    /// Records what a shader's attach found: the types it gave untyped attributes,
    /// which repack what they change, and the attributes it reads, which every
    /// vertex and instance added later must hold. The attach checked the ones the
    /// mesh holds now. Records nothing when another shader has typed one of them
    /// another way meanwhile. A binding that assumed the Model columns sets them
    /// as the instance layout, unless an instance or a layout arrived since. The
    /// layout lands before the reads that need it, so a pack between the two passes.
    pub(crate) fn declare(&self, shader: u64, binding: Binding) -> Result<(), ShaderError> {
        let Binding {
            types: added,
            read,
            columns,
        } = binding;
        if !added.is_empty() {
            let mut types = self.types.write();
            check_conflicts(&types.vertex, &added.vertex)?;
            check_conflicts(&types.instance, &added.instance)?;
            types.vertex.extend(added.vertex.clone());
            types.instance.extend(added.instance.clone());
        }
        let assumed = columns && self.assume_columns();
        #[cfg(test)]
        crate::hooks::run("columns");
        let fresh = self.require(shader, read);
        let vertices = fresh.vertices || !added.vertex.is_empty();
        let instances = fresh.instances || !added.instance.is_empty() || assumed;
        if !added.vertex.is_empty() {
            reset_schema(&self.vertex_schema, &added.vertex);
        }
        if !added.instance.is_empty() {
            reset_schema(&self.instance_schema, &added.instance);
        }
        if vertices {
            self.touch_vertices();
        }
        if instances {
            self.touch_instances();
        }
        Ok(())
    }

    /// Adds `read` to what the shader `shader` reads. Returns, by stream,
    /// whether it names an attribute no attached shader read before, which
    /// the next pack checks every vertex or instance for.
    fn require(&self, shader: u64, read: AttributeTypes) -> Repacked {
        let mut required = self.required.write();
        let unread = |name: &String, stream: fn(&AttributeTypes) -> &HashMap<String, _>| {
            !required
                .values()
                .any(|attached| stream(&attached.reads).contains_key(name))
        };
        let fresh = Repacked {
            vertices: read
                .vertex
                .keys()
                .any(|name| unread(name, |reads| &reads.vertex)),
            instances: read
                .instance
                .keys()
                .any(|name| unread(name, |reads| &reads.instance)),
        };
        let attached = required.entry(shader).or_default();
        attached.reads.vertex.extend(read.vertex);
        attached.reads.instance.extend(read.instance);
        fresh
    }

    /// Sets the Model columns as the instance layout of a mesh
    /// with no instances and no layout, and returns whether it did.
    fn assume_columns(&self) -> bool {
        let insts = self.insts.read();
        let mut schema = self.instance_schema.write();
        if schema.is_some() || !insts.is_empty() {
            return false;
        }
        *schema = Some(model_schema());
        true
    }

    /// Drops what the shader `shader` reads and the layouts it kept,
    /// once that shader no longer holds the mesh.
    pub(crate) fn detach(&self, shader: u64) {
        self.required.write().remove(&shader);
    }

    /// What the shader `shader` reads, while it holds the mesh.
    #[cfg(test)]
    pub(crate) fn reads_of(&self, shader: u64) -> Option<AttributeTypes> {
        let required = self.required.read();
        required.get(&shader).map(|attached| attached.reads.clone())
    }

    /// The attributes some attached shader reads, by stream, with their formats.
    fn read_required(&self) -> AttributeTypes {
        let required = self.required.read();
        let mut union = AttributeTypes::default();
        for attached in required.values() {
            union.vertex.extend(attached.reads.vertex.clone());
            union.instance.extend(attached.reads.instance.clone());
        }
        union
    }

    /// Packs each stream that changed since its last pack, and checks it as it packs:
    /// the indices against the vertices, and each value against its field. A stream
    /// that fails stays unpacked, so every upload fails the same way until it fits.
    ///
    /// It reads the change counts first, then what the shaders read, then the data.
    /// A change that lands after a count was read moves that count again,
    /// so the stream packs once more at the next upload.
    fn ensure_packed(&self) -> Result<Repacked, MeshError> {
        let mut packed = self.packed.write();
        let vertices = self.vertex_changes.load(Ordering::Acquire);
        let instances = self.instance_changes.load(Ordering::Acquire);
        let repacked = Repacked {
            vertices: packed.vertices_at != Some(vertices),
            instances: packed.instances_at != Some(instances),
        };
        let required = self.read_required();
        if repacked.vertices {
            self.pack_vertices(&mut packed, &required.vertex)?;
        }
        if repacked.instances {
            self.pack_instances(&mut packed, &required.instance)?;
        }
        // Only a pack that both streams passed counts, since only it reaches the GPU.
        if repacked.vertices {
            packed.vertices_at = Some(vertices);
        }
        if repacked.instances {
            packed.instances_at = Some(instances);
        }
        Ok(repacked)
    }

    /// Packs the vertices and the list that draws them, under one hold of the vertices,
    /// so the list it checks is the list it packs. A missing layout is derived under its
    /// own lock, so a `declare` either lands before the derive reads `types` or resets it.
    fn pack_vertices(
        &self,
        packed: &mut Packed,
        required: &HashMap<String, wgpu::VertexFormat>,
    ) -> Result<(), MeshError> {
        #[cfg(test)]
        crate::hooks::run("pack");
        let verts = self.verts.read();
        let given = match *self.indices_overridden.read() {
            true => Some(self.indices.read().clone()),
            false => None,
        };
        // The indices draw a triangle list: the ones `set_indices` gave, or one per vertex.
        match &given {
            Some(list) => check_indices(list, verts.len())?,
            None => check_vertices(verts.len())?,
        }
        let mut schema = self.vertex_schema.write();
        if schema.is_none()
            && let Some(first) = verts.first()
        {
            *schema = Some(derive_vertex_schema(first, &self.types.read().vertex));
        }
        let mut bytes = Vec::new();
        let indices = match given {
            // Every vertex in insertion order, for the indices the user gave.
            Some(list) => {
                for (index, vertex) in verts.iter().enumerate() {
                    let layout = schema.as_ref().ok_or(MeshError::NoVertexSchema)?;
                    pack_vertex(&mut bytes, index, vertex, layout, required)?;
                }
                list
            }
            // Equal vertices pack once, and the derived indices point at them.
            None => {
                let mut seen: HashMap<VertexKey, u32> = HashMap::new();
                let mut unique: Vec<(usize, &Vertex)> = Vec::new();
                let mut list: Vec<u32> = Vec::with_capacity(verts.len());
                for (index, vertex) in verts.iter().enumerate() {
                    let next = count(unique.len())?;
                    let slot = *seen.entry(VertexKey::from(vertex)).or_insert(next);
                    if slot == next {
                        unique.push((index, vertex));
                    }
                    list.push(slot);
                }
                for (index, vertex) in unique {
                    let layout = schema.as_ref().ok_or(MeshError::NoVertexSchema)?;
                    pack_vertex(&mut bytes, index, vertex, layout, required)?;
                }
                list
            }
        };
        packed.verts = bytes;
        packed.indices = indices;
        Arc::make_mut(&mut packed.schemas).vertex = schema.clone();
        Ok(())
    }

    /// Packs the instances and counts them, under one hold of the instance list.
    /// An attribute a shader reads from the instances is in the instance layout,
    /// which comes from the first instance or a Model's columns, or no instance holds it.
    fn pack_instances(
        &self,
        packed: &mut Packed,
        required: &HashMap<String, wgpu::VertexFormat>,
    ) -> Result<(), MeshError> {
        let insts = self.insts.read();
        let mut schema = self.instance_schema.write();
        if schema.is_none()
            && let Some(first) = insts.first()
        {
            *schema = Some(derive_instance_schema(first, &self.types.read().instance));
        }
        let fields = schema.as_ref().map(|schema| schema.fields.as_slice());
        check_layout(required, fields.unwrap_or_default(), insts.len())?;
        let mut bytes = Vec::new();
        for (index, instance) in insts.iter().enumerate() {
            let layout = schema.as_ref().ok_or(MeshError::NoInstanceSchema)?;
            pack_instance(&mut bytes, index, instance, layout, required)?;
        }
        packed.insts = bytes;
        packed.rows = insts.len();
        Arc::make_mut(&mut packed.schemas).instance = schema.clone();
        Ok(())
    }

    /// Get Vertex Buffers, Draw counts and the layouts they follow for this mesh,
    /// creating or updating GPU resources on the device of `context` as needed.
    /// Buffers on another context's device give way to new ones.
    pub(crate) fn upload(&self, context: &RenderContext) -> Result<Upload, RendererError> {
        let generation = self.generation();
        if let Some(found) = self.find_cached(context.id, generation) {
            return Ok(found);
        }
        // One upload at a time, and the result it caches names the context it uploaded for.
        let mut gpu = self.gpu.write();
        if let Some(found) = self.find_cached(context.id, generation) {
            return Ok(found);
        }
        let upload = self.upload_vertex_buffers(context, &mut gpu, generation)?;
        *self.cache.write() = Some(Cached {
            owner: context.id,
            upload: upload.clone(),
        });
        Ok(upload)
    }

    /// The cached upload for the context `owner`, unless the mesh changed since `generation`.
    fn find_cached(&self, owner: u64, generation: u64) -> Option<Upload> {
        let cache = self.cache.read();
        let cached = cache.as_ref()?;
        let current = cached.owner == owner && cached.upload.generation == generation;
        current.then(|| cached.upload.clone())
    }

    /// Packs and uploads the mesh into `gpu`. An upload that fails to write a stream
    /// it packed leaves that stream unpacked, so the next one packs and writes it again.
    fn upload_vertex_buffers(
        &self,
        context: &RenderContext,
        gpu: &mut Option<GpuStreams>,
        generation: u64,
    ) -> Result<Upload, RendererError> {
        let repacked = self.ensure_packed()?;
        #[cfg(test)]
        crate::hooks::run("packed");
        let uploaded = self.write_buffers(context, gpu, repacked, generation);
        if uploaded.is_err() {
            self.packed.write().forget(repacked);
        }
        uploaded
    }

    /// Writes what the last pack made into `gpu`, the streams `repacked` names at least.
    fn write_buffers(
        &self,
        context: &RenderContext,
        gpu: &mut Option<GpuStreams>,
        repacked: Repacked,
        generation: u64,
    ) -> Result<Upload, RendererError> {
        let (device, queue) = (&context.device, &context.queue);

        let packed = self.packed.read();
        let (pv, pi, idx) = (&packed.verts, &packed.insts, &packed.indices);

        // The rows come from the pack that made the bytes, so instances without
        // properties count, and a list that shrinks into the old buffer draws its own length.
        let instance_rows = match packed.rows {
            0 => None,
            rows => Some(count(rows)?),
        };
        let index_count = count(idx.len())?;

        // Checked on every upload: a failed one leaves the streams dirty,
        // so data that stays too large fails again instead of drawing old buffers.
        let limit = context.limits.max_buffer_size;
        let sizes = [
            ("the mesh's vertex buffer", pv.len() as u64),
            ("the mesh's index buffer", u64::from(index_count) * 4),
            ("the mesh's instance buffer", pi.len() as u64),
        ];
        for (buffer, size) in sizes {
            check_limit(buffer, "max_buffer_size", size, limit)?;
        }

        // Decide whether we need to recreate GPU buffers. Buffers on
        // another device can't draw here, so they give way to new ones.
        let mut need_new = match gpu.as_ref() {
            None => true,
            Some(g) => g.owner != context.id || g.index_count != index_count || repacked.vertices,
        };

        if let Some(g) = gpu.as_ref() {
            // Instance buffer presence/size changes force (re)creation
            let had_inst = g.instance_buffer.is_some();
            let want_inst = !pi.is_empty();
            if had_inst != want_inst {
                need_new = true;
            } else if want_inst {
                let current_cap = g.instance_buffer_capacity;
                let needed = pi.len() as u64;
                if needed > current_cap {
                    need_new = true;
                }
            }
        }

        if need_new {
            let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Vertex Buffer"),
                size: pv.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Index Buffer"),
                size: (idx.len() as u64) * 4,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&vertex_buffer, 0, pv);
            queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(idx));

            let (instance_buffer, instance_bytes) = if !pi.is_empty() {
                let needed = pi.len() as u64;
                let buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Instance Buffer"),
                    size: needed,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&buf, 0, pi);
                (Some(buf), needed)
            } else {
                (None, 0)
            };

            gpu.replace(GpuStreams {
                owner: context.id,
                vertex_buffer,
                index_buffer,
                index_count,
                instance_buffer,
                instance_buffer_capacity: instance_bytes,
            });
        } else if repacked.instances
            && let Some(buffer) = gpu.as_ref().and_then(|g| g.instance_buffer.as_ref())
        {
            // The vertices didn't change, or the buffers would be new. The instances
            // fit the buffer they have, as the capacity check above found.
            queue.write_buffer(buffer, 0, pi);
        }

        let Some(g) = gpu.as_ref() else {
            return Err(MeshError::NoGpuStreams.into());
        };
        let vertex_buffers = VertexBuffers {
            vertex_buffer: g.vertex_buffer.clone(),
            index_buffer: g.index_buffer.clone(),
            instance_buffer: g.instance_buffer.clone(),
        };

        let override_count = *self.override_instances.read();
        let draw_counts = DrawCounts {
            index_count: g.index_count,
            instance_count: override_count.or(instance_rows).unwrap_or(1),
            instance_rows,
        };

        Ok(Upload {
            buffers: vertex_buffers,
            counts: draw_counts,
            schemas: packed.schemas.clone(),
            generation,
        })
    }
}

pub(crate) struct VertexBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: Option<wgpu::Buffer>,
}

impl Clone for VertexBuffers {
    fn clone(&self) -> Self {
        Self {
            vertex_buffer: self.vertex_buffer.clone(),
            index_buffer: self.index_buffer.clone(),
            instance_buffer: self.instance_buffer.clone(),
        }
    }
}

impl Debug for VertexBuffers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexBuffers")
            .field("vertex_buffer", &self.vertex_buffer)
            .field("index_buffer", &self.index_buffer)
            .field("instance_buffer", &self.instance_buffer)
            .finish()
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct DrawCounts {
    pub index_count: u32,
    pub instance_count: u32,
    /// The instances the mesh holds, or `None` when it holds none.
    pub instance_rows: Option<u32>,
}

impl MeshObject {
    pub(crate) fn vertex_location_map(&self) -> (u32, HashMap<u32, String>) {
        let verts = self.verts.read();
        if let Some(v) = verts.first() {
            // position defaults to 0; properties follow insertion order via stored map
            let pos_loc = 0u32;
            let mut rev: HashMap<u32, String> = HashMap::new();
            for (k, loc) in v.prop_locations.iter() {
                rev.insert(*loc, k.clone());
            }
            (pos_loc, rev)
        } else {
            (0u32, HashMap::new())
        }
    }
    pub(crate) fn instance_location_map(&self) -> HashMap<u32, String> {
        let insts = self.insts.read();
        if let Some(i) = insts.first() {
            let mut rev: HashMap<u32, String> = HashMap::new();
            for (k, loc) in i.prop_locations.iter() {
                rev.insert(*loc, k.clone());
            }
            rev
        } else {
            HashMap::new()
        }
    }
}

/// The names of the four `vec4<f32>` columns of a `Model`'s transform.
const MODEL_COLUMNS: [&str; 4] = ["model_0", "model_1", "model_2", "model_3"];

/// The instance layout of a `Model`: the four `vec4<f32>` columns of its transform.
pub(crate) fn model_schema() -> VertexSchema {
    let fields = MODEL_COLUMNS
        .into_iter()
        .map(|name| Field::new(name, wgpu::VertexFormat::Float32x4))
        .collect();
    VertexSchema::new(fields)
}

fn derive_vertex_schema(
    vertex: &Vertex,
    types: &HashMap<String, wgpu::VertexFormat>,
) -> VertexSchema {
    let position = match vertex.dimensions {
        0..=2 => wgpu::VertexFormat::Float32x2,
        _ => wgpu::VertexFormat::Float32x3,
    };
    let mut fields = vec![Field::new(POSITION, position)];
    fields.extend(derive_fields(&vertex.properties, types));
    VertexSchema::new(fields)
}

fn derive_instance_schema(
    instance: &Instance,
    types: &HashMap<String, wgpu::VertexFormat>,
) -> VertexSchema {
    VertexSchema::new(derive_fields(&instance.properties, types))
}

/// One field per value in key order, in the format each value resolves to.
fn derive_fields(
    properties: &HashMap<String, VertexData>,
    types: &HashMap<String, wgpu::VertexFormat>,
) -> Vec<Field> {
    let mut values: Vec<(&String, &VertexData)> = properties.iter().collect();
    values.sort_by(|left, right| left.0.cmp(right.0));
    values
        .into_iter()
        .map(|(name, value)| Field::new(name, value.resolve(types.get(name).copied())))
        .collect()
}

/// Fails on the first attribute `added` types another way than `known` does.
fn check_conflicts(
    known: &HashMap<String, wgpu::VertexFormat>,
    added: &HashMap<String, wgpu::VertexFormat>,
) -> Result<(), ShaderError> {
    for (name, format) in added {
        if let Some(fixed) = known.get(name)
            && fixed != format
        {
            return Err(ShaderError::AttributeConflict {
                name: name.clone(),
                fixed: describe(*fixed).to_string(),
                declared: describe(*format).to_string(),
            });
        }
    }
    Ok(())
}

/// Drops a schema that lists one of `names`, so the next pack derives it with their types.
fn reset_schema(
    schema: &RwLock<Option<VertexSchema>>,
    names: &HashMap<String, wgpu::VertexFormat>,
) {
    let mut schema = schema.write();
    let listed = schema.as_ref().is_some_and(|schema| {
        schema
            .fields
            .iter()
            .any(|field| names.contains_key(&field.name))
    });
    if listed {
        *schema = None;
    }
}

/// Packs vertex `index` in the mesh layout `schema`. A field an attached shader reads
/// (`required`) must hold a value, and any other field the vertex leaves out packs as zeros.
fn pack_vertex(
    out: &mut Vec<u8>,
    index: usize,
    v: &Vertex,
    schema: &VertexSchema,
    required: &HashMap<String, wgpu::VertexFormat>,
) -> Result<(), MeshError> {
    if v.properties.contains_key(POSITION) {
        return Err(MeshError::InvalidVertex(explain_position(index)));
    }
    for f in schema.fields.iter() {
        match f.name.as_str() {
            POSITION => {
                let [x, y, z, _] = v.position.0.to_array();
                match (v.dimensions, f.fmt) {
                    (0..=2, wgpu::VertexFormat::Float32x2) => {
                        out.extend_from_slice(bytemuck::cast_slice(&[x, y]));
                    }
                    (3.., wgpu::VertexFormat::Float32x3) => {
                        out.extend_from_slice(bytemuck::cast_slice(&[x, y, z]));
                    }
                    (dimensions, format) => {
                        return Err(MeshError::LayoutMismatch(format!(
                            "a vertex has a {dimensions}D position, but the mesh layout has {format:?}"
                        )));
                    }
                }
            }
            name => match v.properties.get(name) {
                Some(value) => value.write(f.fmt, out).map_err(|misfit| {
                    MeshError::InvalidVertex(misfit.explain(name, &format!("vertex {index}")))
                })?,
                None if required.contains_key(name) => {
                    let place = format!("vertex {index}");
                    return Err(MeshError::InvalidVertex(explain_missing(f, &place)));
                }
                None => out.extend(std::iter::repeat_n(0u8, f.size as usize)),
            },
        }
    }
    Ok(())
}

/// Packs instance `index` in the instance layout `schema`, as `pack_vertex` packs a vertex.
fn pack_instance(
    out: &mut Vec<u8>,
    index: usize,
    i: &Instance,
    schema: &VertexSchema,
    required: &HashMap<String, wgpu::VertexFormat>,
) -> Result<(), MeshError> {
    for f in schema.fields.iter() {
        match i.properties.get(&f.name) {
            Some(value) => value.write(f.fmt, out).map_err(|misfit| {
                MeshError::InvalidInstance(misfit.explain(&f.name, &format!("instance {index}")))
            })?,
            None if required.contains_key(&f.name) => {
                let place = format!("instance {index}");
                return Err(MeshError::InvalidInstance(explain_missing(f, &place)));
            }
            None => out.extend(std::iter::repeat_n(0u8, f.size as usize)),
        }
    }
    Ok(())
}

/// Why the item at `place` can't leave out `field`, worded as the attach words it.
fn explain_missing(field: &Field, place: &str) -> String {
    format!(
        "{}: expected {}, got no value at {place}",
        field.name,
        describe(field.fmt)
    )
}

/// Refuses instances that can't hold an attribute a shader reads from them: one missing
/// from the instance layout, which comes from the first instance or a Model's columns.
fn check_layout(
    required: &HashMap<String, wgpu::VertexFormat>,
    fields: &[Field],
    instances: usize,
) -> Result<(), MeshError> {
    let missing = required
        .iter()
        .filter(|(name, _)| !fields.iter().any(|field| &field.name == *name))
        .min_by_key(|(name, _)| name.as_str());
    let Some((name, format)) = missing else {
        return Ok(());
    };
    let found = match instances {
        0 => "no instances".to_string(),
        _ => "no value at instance 0".to_string(),
    };
    Err(MeshError::InvalidInstance(format!(
        "{name}: expected {}, got {found}",
        describe(*format)
    )))
}

/// Refuses a mesh without `set_indices` whose vertices, drawn in threes,
/// end partway through a triangle.
fn check_vertices(vertices: usize) -> Result<(), MeshError> {
    match vertices.is_multiple_of(3) {
        true => Ok(()),
        false => Err(MeshError::VertexCount(vertices)),
    }
}

/// Refuses an index list that ends partway through a triangle,
/// or that points past the last of `vertices`.
fn check_indices(indices: &[u32], vertices: usize) -> Result<(), MeshError> {
    if !indices.len().is_multiple_of(3) {
        return Err(MeshError::IndexCount(indices.len()));
    }
    match indices.iter().find(|index| **index as usize >= vertices) {
        Some(index) => Err(MeshError::IndexRange {
            index: *index,
            vertices,
        }),
        None => Ok(()),
    }
}

/// `length` as a 32-bit count, the width the GPU draws indices and instances with.
fn count(length: usize) -> Result<u32, MeshError> {
    Ok(u32::try_from(length)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The streams the next upload packs again.
    fn stale(mesh: &Mesh) -> Repacked {
        let packed = mesh.object.packed.read();
        Repacked {
            vertices: packed.vertices_at
                != Some(mesh.object.vertex_changes.load(Ordering::Acquire)),
            instances: packed.instances_at
                != Some(mesh.object.instance_changes.load(Ordering::Acquire)),
        }
    }

    // Story: the Model columns are told apart by name and format alone, so the
    // per-frame check that skips a Model's mesh builds no layout to compare with.
    #[test]
    fn the_model_columns_match_by_name_and_format() {
        let floats = |names: &[&str]| {
            let fields = names
                .iter()
                .map(|name| Field::new(name, wgpu::VertexFormat::Float32x4));
            VertexSchema::new(fields.collect())
        };
        let integers = VertexSchema::new(
            MODEL_COLUMNS
                .map(|name| Field::new(name, wgpu::VertexFormat::Uint32x4))
                .to_vec(),
        );

        assert!(model_schema().is_model());
        assert!(floats(&MODEL_COLUMNS).is_model());
        assert!(!floats(&MODEL_COLUMNS[..3]).is_model(), "three columns");
        assert!(!floats(&["model_0", "model_1", "model_2", "tint"]).is_model());
        assert!(!integers.is_model(), "another format");
    }

    #[test]
    fn derives_schema_and_stride_for_vertex() {
        let v = Vertex::new([0.1f32, 0.2f32])
            .set("uv", [0.3f32, 0.4])
            .set("color", [1.0f32, 1.0, 1.0, 1.0]);
        let schema = derive_vertex_schema(&v, &HashMap::new());
        // position (vec2), color (vec4), uv (vec2) sorted by key name
        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.fields[0].name, "position");
        assert_eq!(schema.fields[0].fmt, wgpu::VertexFormat::Float32x2);
        assert_eq!(schema.fields[1].name, "color");
        assert_eq!(schema.fields[1].fmt, wgpu::VertexFormat::Float32x4);
        assert_eq!(schema.fields[2].name, "uv");
        assert_eq!(schema.fields[2].fmt, wgpu::VertexFormat::Float32x2);
        assert_eq!(schema.stride, 8 + 16 + 8);
    }

    // Story: one key with two shapes would misalign every later vertex, so packing
    // rejects it instead, and the same goes for a position of the wrong dimension.
    #[test]
    fn ensure_packed_rejects_values_that_differ_from_the_layout() {
        let mesh = Mesh::new();
        mesh.add_vertices([
            Vertex::new([0.0f32, 0.0, 0.0]).set("uv", [0.0f32, 0.0, 0.0]),
            Vertex::new([1.0f32, 0.0, 0.0]).set("uv", [1.0f32, 0.0]),
            Vertex::new([0.0f32, 1.0, 0.0]).set("uv", [0.0f32, 1.0, 0.0]),
        ]);
        match mesh.object.ensure_packed() {
            Err(MeshError::InvalidVertex(text)) => {
                assert_eq!(text, "uv: expected vec3<f32>, got vec2<f32> at vertex 1");
            }
            other => panic!("expected an invalid vertex, got {other:?}"),
        }

        let mesh = Mesh::new();
        mesh.add_vertices([
            Vertex::new([0.0f32, 0.0, 0.0]),
            Vertex::new([1.0f32, 0.0]),
            Vertex::new([0.0f32, 1.0, 0.0]),
        ]);
        assert!(matches!(
            mesh.object.ensure_packed(),
            Err(MeshError::LayoutMismatch(_))
        ));

        let mesh = Mesh::new();
        mesh.add_vertices([[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]]);
        mesh.add_instances([
            Instance::new().set("offset", [0.0f32, 0.0]),
            Instance::new().set("offset", 1.0f32),
        ]);
        match mesh.object.ensure_packed() {
            Err(MeshError::InvalidInstance(text)) => {
                assert_eq!(text, "offset: expected vec2<f32>, got f32 at instance 1");
            }
            other => panic!("expected an invalid instance, got {other:?}"),
        }
    }

    // Story: a vertex that sets a property named position fails the pack, since its
    // position comes from Vertex::new, and the mesh layout would list it twice.
    #[test]
    fn ensure_packed_refuses_a_position_property() {
        let mesh = Mesh::new();
        mesh.add_vertices(
            [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
                .map(|corner| Vertex::new(corner).set("position", [5.0, 5.0])),
        );

        match mesh.object.ensure_packed() {
            Err(MeshError::InvalidVertex(text)) => assert_eq!(
                text,
                "vertex 0 sets a property named 'position', and a vertex takes its position from Vertex::new"
            ),
            other => panic!("expected an invalid vertex, got {other:?}"),
        }
    }

    // Story: vertex, index and instance data past the device's buffer limit fail
    // the upload with an error that names the buffer, before any buffer is made,
    // and fail again on the next upload while the data stays too large.
    #[test]
    fn uploads_stay_within_the_buffer_limit() {
        pollster::block_on(async move {
            let (device, queue) = crate::renderer::platform::all::request_small_device(65536).await;
            let context = RenderContext::new(device, queue);
            let corners = (0..8193).map(|index| [index as f32, 0.0]);
            let vertices = Mesh::new();
            vertices.add_vertices(corners);
            let indices = Mesh::new();
            indices.add_vertices([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
            indices.set_indices((0..16386).map(|index| index % 3));
            let instances = Mesh::new();
            instances.add_vertices([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);
            instances.add_instances((0..4097).map(|_| Instance::new().set("tint", [1.0; 4])));

            let message = |mesh: &Mesh| match mesh.object.upload(&context) {
                Err(error) => error.to_string(),
                Ok(_) => panic!("expected an error"),
            };

            assert_eq!(
                message(&vertices),
                "the mesh's vertex buffer needs max_buffer_size of at least 65544, and the device has 65536"
            );
            assert_eq!(
                message(&vertices),
                "the mesh's vertex buffer needs max_buffer_size of at least 65544, and the device has 65536",
                "the next upload"
            );
            assert_eq!(
                message(&indices),
                "the mesh's index buffer needs max_buffer_size of at least 65544, and the device has 65536"
            );
            assert_eq!(
                message(&instances),
                "the mesh's instance buffer needs max_buffer_size of at least 65552, and the device has 65536"
            );
        });
    }

    // Story: the mesh builds a shader's layouts once and keeps them until it
    // changes, and a shader that lets the mesh go leaves no layouts behind.
    #[test]
    fn layouts_stay_until_the_mesh_changes() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("texture target");
            let shader = crate::Shader::new(
                "@vertex fn vs_main(@location(0) p: vec2<f32>) -> @builtin(position) vec4<f32> { return vec4<f32>(p, 0.0, 1.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
            )
            .expect("shader");
            let mesh = Mesh::new();
            mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
            shader.add_mesh(&mesh).expect("attach");
            renderer.render(&shader, &target).expect("render");
            let id = shader.object.id;
            let refuse = || Err(RendererError::Error("rebuilt".into()));
            let fetch = |mesh: &Mesh| {
                mesh.object
                    .fetch_layouts(id, mesh.object.generation(), refuse)
            };

            let kept = fetch(&mesh);
            mesh.add_vertices([[0.0, 0.0], [0.1, 0.0], [0.0, 0.1]]);
            let changed = fetch(&mesh);
            renderer.render(&shader, &target).expect("render again");
            shader.remove_mesh(&mesh);
            let entries = mesh.object.required.read().len();
            let released = fetch(&mesh);

            assert!(kept.is_ok(), "a still mesh keeps its layouts: {kept:?}");
            assert!(changed.is_err(), "a changed mesh builds them again");
            assert_eq!(entries, 0, "a shader that let go keeps no entry");
            assert!(released.is_err(), "and no layouts");
        });
    }

    // Story: a shader draws a mesh with the layouts of the pack its buffers came
    // from, so another shader that types an attribute meanwhile doesn't fail the frame.
    #[test]
    fn layouts_follow_the_pack_the_buffers_came_from() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("texture target");
            let reads_color = crate::Shader::new(
                "@vertex fn vs_main(@location(0) p: vec2<f32>, @location(1) color: vec4<f32>) -> @builtin(position) vec4<f32> { return vec4<f32>(p, 0.0, color.a); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
            )
            .expect("shader");
            let reads_weight = crate::Shader::new(
                "@vertex fn vs_main(@location(0) p: vec2<f32>, @location(1) weight: u32) -> @builtin(position) vec4<f32> { return vec4<f32>(p, f32(weight), 1.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
            )
            .expect("shader");
            let mesh = Mesh::new();
            mesh.add_vertices(
                [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]
                    .map(|corner| Vertex::new(corner).set("color", [1.0; 4]).set("weight", 2)),
            );
            reads_color.add_mesh(&mesh).expect("attach");
            let late = mesh.clone();
            let attached = std::rc::Rc::new(std::cell::RefCell::new(None));
            let seen = attached.clone();
            // The weight shader types the plain numbers as u32, which resets the
            // mesh layout after the color shader's upload and before its layouts.
            crate::hooks::once("uploaded", move || {
                *seen.borrow_mut() = Some(reads_weight.add_mesh(&late).map(drop));
            });

            let rendered = renderer.render(&reads_color, &target);

            let attached = attached.borrow_mut().take().expect("an attach in between");
            assert!(attached.is_ok(), "{attached:?}");
            assert!(rendered.is_ok(), "{rendered:?}");
        });
    }

    // Story: the quick view checks the mesh before each draw instead of recording
    // what it reads, so a shader of the user's own reads only what it declares.
    #[test]
    fn the_quick_view_leaves_other_shaders_their_rules() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("texture target");
            let mesh = Mesh::new();
            let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
            mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));
            renderer.render(&mesh, &target).expect("quick view");
            let own = crate::Shader::new(
                "@vertex fn vs_main(@location(0) p: vec2<f32>) -> @builtin(position) vec4<f32> { return vec4<f32>(p, 0.0, 1.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
            )
            .expect("shader");
            own.add_mesh(&mesh).expect("attach");

            mesh.add_vertices(corners.map(Vertex::new));
            let drawn = renderer.render(&own, &target);
            let previewed = renderer.render(&mesh, &target);

            assert!(drawn.is_ok(), "the own shader reads no color: {drawn:?}");
            let error = previewed
                .expect_err("the quick view reads color")
                .to_string();
            assert!(error.contains("color"), "{error}");
            assert!(error.contains("vertex 3"), "{error}");
        });
    }

    // Story: every render of a mesh the quick view refuses says why,
    // concurrent ones included, rather than the error for a shader with no mesh.
    #[test]
    fn every_render_of_a_refused_quick_view_says_why() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("texture target");
            let mesh = Mesh::new();
            mesh.add_vertices(
                [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]
                    .map(|corner| Vertex::new(corner).set("position", [5.0, 5.0])),
            );
            mesh.passes();
            let pass = crate::Pass {
                object: mesh.pass.clone(),
            };

            for render in ["first", "second"] {
                let error = renderer.render(&pass, &target).expect_err(render);
                let text = error.to_string();
                assert!(
                    text.contains("property named 'position'"),
                    "{render}: {text}"
                );
            }
        });
    }

    // Story: a pass that requires a mesh draws it through the quick view, which
    // checks the mesh on each render of that pass, so a vertex added later without
    // the color the quick view reads fails the render instead of drawing zeros.
    #[test]
    fn a_required_quick_view_checks_each_render() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("texture target");
            let mesh = Mesh::new();
            let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
            mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));
            let pass = crate::Pass::new("main");
            pass.require(&mesh).expect("require");
            renderer.render(&pass, &target).expect("first render");

            mesh.add_vertices(corners.map(Vertex::new));
            let error = renderer
                .render(&pass, &target)
                .expect_err("vertex 3 has no color")
                .to_string();

            assert!(error.contains("color"), "{error}");
            assert!(error.contains("vertex 3"), "{error}");
        });
    }

    // Story: a mesh the quick view refused when a pass required it draws
    // once it fits, instead of failing as a shader with no mesh.
    #[test]
    fn a_required_quick_view_draws_a_mesh_fixed_later() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("texture target");
            let mesh = Mesh::new();
            let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
            mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));
            // An instance color is read first, and two floats don't fit the vec4.
            mesh.add_instance(Instance::new().set("color", [1.0f32, 0.0]));
            let pass = crate::Pass::new("main");
            pass.require(&mesh).expect("require");

            let refused = renderer.render(&pass, &target);
            mesh.clear_instances();
            let fixed = renderer.render(&pass, &target);

            let error = refused.expect_err("the instance color").to_string();
            assert!(error.contains("color"), "{error}");
            assert!(fixed.is_ok(), "{fixed:?}");
        });
    }

    // Story: a vertex counts as a change before any other thread can see it, so a
    // vertex the quick view packs after its check is checked by the same render.
    #[test]
    fn a_vertex_added_during_the_quick_view_pack_is_checked() {
        pollster::block_on(async move {
            let renderer = crate::Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("texture target");
            let mesh = Mesh::new();
            let corners = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]];
            mesh.add_vertices(corners.map(|corner| Vertex::new(corner).set("color", [1.0; 4])));
            mesh.set_indices([0, 1, 2]);
            let late = mesh.clone();
            let (visible, seen) = std::sync::mpsc::channel();
            let (finish, finished) = std::sync::mpsc::channel::<()>();
            let adder = Arc::new(parking_lot::Mutex::new(None));
            let slot = adder.clone();
            // Another thread adds a vertex without color and holds still once other
            // threads can see it, while this one packs.
            crate::hooks::once("pack", move || {
                let thread = std::thread::spawn(move || {
                    crate::hooks::once("added", move || {
                        let _ = visible.send(());
                        let _ = finished.recv();
                    });
                    late.add_vertex([0.0, 0.0]);
                });
                let _ = seen.recv_timeout(std::time::Duration::from_secs(10));
                *slot.lock() = Some(thread);
            });

            let rendered = renderer.render(&mesh, &target);
            let _ = finish.send(());
            if let Some(thread) = adder.lock().take() {
                thread.join().expect("the adder thread");
            }

            let error = rendered.expect_err("vertex 3 has no color").to_string();
            assert!(error.contains("vertex 3"), "{error}");
        });
    }

    // Story: the quick view reads the first vertex in a statement of its own,
    // so it holds no vertex guard while the shader checks the vertices again.
    // Under a fair lock, a writer waiting between the two reads would hang both.
    #[test]
    fn the_quick_view_holds_no_vertex_guard_while_it_attaches() {
        let mesh = Mesh::new();
        mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
        let object = mesh.object.clone();
        let free = std::rc::Rc::new(std::cell::Cell::new(false));
        let seen = free.clone();
        crate::hooks::once("preview", move || {
            seen.set(object.verts.try_write().is_some());
        });

        mesh.passes();

        assert!(
            free.get(),
            "the vertices were locked while the mesh attached"
        );
    }

    // Story: a vertex or an index that arrives while an upload runs is checked by
    // the upload that packs it, so a partial triangle never reaches the GPU.
    #[test]
    fn a_change_during_the_upload_is_checked() {
        pollster::block_on(async move {
            let instance = crate::renderer::platform::all::create_instance().await;
            let adapter = crate::renderer::platform::all::request_adapter(&instance, None)
                .await
                .expect("adapter");
            let (device, queue) = crate::renderer::platform::all::request_device(&adapter)
                .await
                .expect("device");
            let context = RenderContext::new(device, queue);

            let derived = Mesh::new();
            derived.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
            let late = derived.clone();
            crate::hooks::once("pack", move || late.add_vertex([0.5, 0.5]));
            let error = derived.object.upload(&context).expect_err("4 vertices");
            assert_eq!(
                error.to_string(),
                RendererError::from(MeshError::VertexCount(4)).to_string(),
                "derived indices"
            );

            let given = Mesh::new();
            given.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
            given.set_indices([0, 1, 2]);
            let late = given.clone();
            crate::hooks::once("pack", move || late.set_indices([0, 1, 2, 0]));
            let error = given.object.upload(&context).expect_err("4 indices");
            assert_eq!(
                error.to_string(),
                RendererError::from(MeshError::IndexCount(4)).to_string(),
                "given indices"
            );
        });
    }

    // Story: an instance added after the pack and before the write stays unpacked,
    // and the upload draws the rows its own pack made.
    #[test]
    fn an_instance_added_during_the_upload_packs_next_time() {
        pollster::block_on(async move {
            let instance = crate::renderer::platform::all::create_instance().await;
            let adapter = crate::renderer::platform::all::request_adapter(&instance, None)
                .await
                .expect("adapter");
            let (device, queue) = crate::renderer::platform::all::request_device(&adapter)
                .await
                .expect("device");
            let context = RenderContext::new(device, queue);
            let mesh = Mesh::new();
            mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
            let tint = || Instance::new().set("tint", [1.0; 4]);
            mesh.add_instances([tint(), tint(), tint()]);
            mesh.object.upload(&context).expect("first upload");
            mesh.clear_instances();
            mesh.add_instances([tint(), tint()]);

            let late = mesh.clone();
            crate::hooks::once("packed", move || late.add_instance(tint()));
            let during = mesh.object.upload(&context).expect("second upload").counts;
            let after = mesh.object.upload(&context).expect("third upload").counts;

            assert_eq!(during.instance_rows, Some(2), "the rows the pack made");
            assert_eq!(
                after.instance_rows,
                Some(3),
                "the late instance packs next time"
            );
            assert_eq!(mesh.object.packed.read().rows, 3);
        });
    }

    // Story: a Model attach sets the Model columns before it records the columns
    // as reads, so a render of the mesh's other shaders that packs in between passes.
    #[test]
    fn a_pack_during_a_model_attach_finds_the_columns() {
        pollster::block_on(async move {
            let instance = crate::renderer::platform::all::create_instance().await;
            let adapter = crate::renderer::platform::all::request_adapter(&instance, None)
                .await
                .expect("adapter");
            let (device, queue) = crate::renderer::platform::all::request_device(&adapter)
                .await
                .expect("device");
            let context = RenderContext::new(device, queue);
            let mesh = Mesh::new();
            mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);
            let mut read = AttributeTypes::default();
            for field in model_schema().fields {
                read.instance.insert(field.name, field.fmt);
            }
            let binding = Binding {
                read,
                columns: true,
                ..Default::default()
            };

            let object = mesh.object.clone();
            let packed = std::rc::Rc::new(std::cell::RefCell::new(None));
            let seen = packed.clone();
            crate::hooks::once("columns", move || {
                let upload = object.upload(&context).map(drop);
                *seen.borrow_mut() = Some(upload);
            });
            mesh.object.declare(1, binding).expect("declare");

            let upload = packed.borrow_mut().take().expect("a pack in between");
            assert!(upload.is_ok(), "{upload:?}");
        });
    }

    // Story: the cached upload names the context it belongs to, so after two
    // contexts take turns, the cache and the buffers belong to the last one.
    #[test]
    fn the_cache_belongs_to_the_context_that_filled_it() {
        pollster::block_on(async move {
            let mut contexts = Vec::new();
            for _ in 0..2 {
                let instance = crate::renderer::platform::all::create_instance().await;
                let adapter = crate::renderer::platform::all::request_adapter(&instance, None)
                    .await
                    .expect("adapter");
                let (device, queue) = crate::renderer::platform::all::request_device(&adapter)
                    .await
                    .expect("device");
                contexts.push(RenderContext::new(device, queue));
            }
            let mesh = Mesh::new();
            mesh.add_vertices([[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]);

            for context in [&contexts[0], &contexts[1], &contexts[0], &contexts[1]] {
                mesh.object.upload(context).expect("buffers");
                let cache = mesh.object.cache.read();
                let gpu = mesh.object.gpu.read();
                assert_eq!(cache.as_ref().map(|cached| cached.owner), Some(context.id));
                assert_eq!(gpu.as_ref().map(|streams| streams.owner), Some(context.id));
            }
        });
    }

    // Story: a plain number packs as a float until a shader types it, then
    // the mesh repacks it in that type and keeps it for the mesh's lifetime.
    #[test]
    fn untyped_attributes_pack_as_floats_until_declared() {
        let mesh = Mesh::new();
        mesh.add_vertices([
            Vertex::new([0.0f32, 0.0]).set("id", 3),
            Vertex::new([1.0f32, 0.0]).set("id", 4),
            Vertex::new([0.0f32, 1.0]).set("id", 5),
        ]);
        let field = |mesh: &Mesh| -> (wgpu::VertexFormat, Vec<u8>, Vec<u8>) {
            mesh.object.ensure_packed().expect("pack");
            let schema = mesh.object.vertex_schema.read().clone().expect("schema");
            let bytes = mesh.object.packed.read().verts.clone();
            let stride = schema.stride as usize;
            (
                schema.fields[1].fmt,
                bytes[8..12].to_vec(),
                bytes[stride + 8..stride + 12].to_vec(),
            )
        };

        let (format, first, second) = field(&mesh);
        assert_eq!(format, wgpu::VertexFormat::Float32);
        assert_eq!(first, 3.0f32.to_ne_bytes());
        assert_eq!(second, 4.0f32.to_ne_bytes());

        let mut added = AttributeTypes::default();
        added
            .vertex
            .insert("id".to_string(), wgpu::VertexFormat::Uint32);
        let binding = Binding {
            types: added,
            ..Default::default()
        };
        mesh.object.declare(1, binding).expect("declare");
        assert!(mesh.object.vertex_schema.read().is_none());
        assert!(stale(&mesh).vertices);

        let (format, first, second) = field(&mesh);
        assert_eq!(format, wgpu::VertexFormat::Uint32);
        assert_eq!(first, 3u32.to_ne_bytes());
        assert_eq!(second, 4u32.to_ne_bytes());

        let mut conflict = AttributeTypes::default();
        conflict
            .vertex
            .insert("id".to_string(), wgpu::VertexFormat::Float32);
        let binding = Binding {
            types: conflict,
            ..Default::default()
        };
        let error = mesh.object.declare(1, binding).expect_err("conflict");
        assert_eq!(
            error.to_string(),
            "Mesh attribute id is u32 on this mesh, but the shader declares f32"
        );
        assert_eq!(
            mesh.object.types.read().vertex.get("id"),
            Some(&wgpu::VertexFormat::Uint32)
        );
    }

    // Story: attaching to shaders that agree on every type keeps the packed
    // bytes and the GPU buffers, since nothing about them changes.
    #[test]
    fn declaring_nothing_keeps_the_gpu_cache() {
        pollster::block_on(async move {
            let wgsl = r#"
@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) id: u32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(pos, f32(id), 1.0);
}
@fragment
fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
            let mesh = Mesh::new();
            mesh.add_vertices([
                Vertex::new([-1.0f32, -1.0]).set("id", 1),
                Vertex::new([3.0f32, -1.0]).set("id", 1),
                Vertex::new([-1.0f32, 3.0]).set("id", 1),
            ]);
            let first = crate::Shader::new(wgsl).expect("shader");
            first.add_mesh(&mesh).expect("first attach");

            let instance = crate::renderer::platform::all::create_instance().await;
            let adapter = crate::renderer::platform::all::request_adapter(&instance, None)
                .await
                .expect("adapter");
            let (device, queue) = crate::renderer::platform::all::request_device(&adapter)
                .await
                .expect("device");
            let context = RenderContext::new(device, queue);
            mesh.object.upload(&context).expect("buffers");
            let cached = |mesh: &Mesh| {
                let generation = mesh.object.generation.load(Ordering::Acquire);
                mesh.object.find_cached(context.id, generation).is_some()
            };
            assert!(cached(&mesh));

            let second = crate::Shader::new(wgsl).expect("shader");
            second.add_mesh(&mesh).expect("second attach");
            first.validate_mesh(&mesh).expect("validate again");
            assert!(cached(&mesh));
            assert!(!stale(&mesh).vertices);
        });
    }

    #[test]
    fn ensure_packed_deduplicates_and_indices_track() {
        let mesh = Mesh::new();
        use crate::mesh::Vertex;
        let a = Vertex::new([0.0f32, 0.0]).set("uv", [0.0f32, 0.0]);
        let b = Vertex::new([1.0f32, 0.0]).set("uv", [1.0f32, 0.0]);
        let c = Vertex::new([0.0f32, 1.0]).set("uv", [0.0f32, 1.0]);
        mesh.add_vertices([a.clone(), b.clone(), c.clone(), a.clone(), c.clone(), b]);

        // Force packing and inspect internals
        mesh.object.ensure_packed().expect("pack");
        let stride = mesh.object.vertex_schema.read().as_ref().unwrap().stride;
        let pv = mesh.object.packed.read().verts.clone();
        let idx = mesh.object.packed.read().indices.clone();
        // Unique verts should be a, b, c => 3 * stride bytes
        assert_eq!(pv.len() as u64, 3 * stride);
        // Indices mirror [a,b,c,a,c,b] => [0,1,2,0,2,1]
        assert_eq!(idx, vec![0, 1, 2, 0, 2, 1]);
    }

    #[test]
    fn instance_packing_override_and_vertex_buffers() {
        pollster::block_on(async move {
            let mesh = Mesh::new();
            use crate::mesh::{Instance, Vertex};
            mesh.add_vertices([
                Vertex::new([-0.5f32, -0.5f32]),
                Vertex::new([0.5f32, -0.5f32]),
                Vertex::new([0.0f32, 0.5f32]),
            ]);
            let inst = Instance::new().set("id", 7u32);
            mesh.add_instances([inst.clone(), inst]);

            // Build GPU buffers
            let instance = crate::renderer::platform::all::create_instance().await;
            let adapter = crate::renderer::platform::all::request_adapter(&instance, None)
                .await
                .expect("adapter");
            let (device, queue) = crate::renderer::platform::all::request_device(&adapter)
                .await
                .expect("device");
            let context = RenderContext::new(device, queue);

            // Without override, instance_count == 2 (matches buffer)
            let counts = mesh.object.upload(&context).expect("vb").counts;
            assert_eq!(counts.instance_count, 2);

            // With override, instance_count == 5
            mesh.set_instance_count(5);
            let counts2 = mesh.object.upload(&context).expect("vb2").counts;
            assert_eq!(counts2.instance_count, 5);

            // Adding an instance auto-clears the override (otherwise the GPU would
            // draw past the actual buffer end and read garbage). count falls back
            // to the new buffer length (3).
            mesh.add_instance(Instance::new().set("id", 8u32));
            let counts3 = mesh.object.upload(&context).expect("vb3").counts;
            assert_eq!(counts3.instance_count, 3);

            // clear_instances resets everything: empty buffer + no override → 1.
            mesh.set_instance_count(99);
            mesh.clear_instances();
            let counts4 = mesh.object.upload(&context).expect("vb4").counts;
            assert_eq!(counts4.instance_count, 1);
        });
    }

    #[test]
    fn renderable_passes_creates_shader_from_first_vertex() {
        let mesh = Mesh::new();
        use crate::mesh::Vertex;
        mesh.add_vertices([
            Vertex::new([-0.5f32, -0.5f32]),
            Vertex::new([0.5f32, -0.5f32]),
            Vertex::new([0.0f32, 0.5f32]),
        ]);
        let passes = mesh.passes();
        assert_eq!(passes.len(), 1);
    }

    #[test]
    fn set_indices_preserves_non_deduped_vertices() {
        let mesh = Mesh::new();
        use crate::mesh::Vertex;
        // Three vertices with fully identical attributes — the auto path
        // would collapse these into a single unique vertex.
        let v = Vertex::new([0.0f32, 0.0]).set("uv", [0.5f32, 0.5]);
        mesh.add_vertex(v.clone());
        mesh.add_vertex(v.clone());
        mesh.add_vertex(v);
        mesh.set_indices([0u32, 1, 2]);

        mesh.object.ensure_packed().expect("pack");
        let stride = mesh.object.vertex_schema.read().as_ref().unwrap().stride;
        let pv = mesh.object.packed.read().verts.clone();
        let idx = mesh.object.packed.read().indices.clone();
        // No dedup: 3 vertices worth of bytes, and indices preserved verbatim.
        assert_eq!(pv.len() as u64, 3 * stride);
        assert_eq!(idx, vec![0, 1, 2]);
    }

    #[test]
    fn clear_indices_restores_auto_derivation() {
        let mesh = Mesh::new();
        use crate::mesh::Vertex;
        let a = Vertex::new([0.0f32, 0.0]).set("uv", [0.0f32, 0.0]);
        let b = Vertex::new([1.0f32, 0.0]).set("uv", [1.0f32, 0.0]);
        mesh.add_vertices([a.clone(), b.clone(), a.clone()]);

        // Switch to override mode with explicit indices.
        mesh.set_indices([0u32, 1, 2]);
        mesh.object.ensure_packed().expect("pack override");
        let stride = mesh.object.vertex_schema.read().as_ref().unwrap().stride;
        assert_eq!(mesh.object.packed.read().verts.len() as u64, 3 * stride);
        assert_eq!(mesh.object.packed.read().indices, vec![0, 1, 2]);

        // Clear and re-pack: dedup should collapse `a` and produce derived indices.
        mesh.clear_indices();
        mesh.object.ensure_packed().expect("pack auto");
        assert_eq!(mesh.object.packed.read().verts.len() as u64, 2 * stride);
        assert_eq!(mesh.object.packed.read().indices, vec![0, 1, 0]);
    }

    // Story: a triangle drawn from explicit indices renders without error.
    #[test]
    fn render_with_mesh_explicit_indices() {
        pollster::block_on(async move {
            let renderer = crate::renderer::Renderer::new();
            let target = renderer
                .create_texture_target([16u32, 16u32])
                .await
                .expect("texture target");

            let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@location(0) pos: vec2<f32>) -> VOut {
  var out: VOut;
  out.pos = vec4<f32>(pos, 0.0, 1.0);
  return out;
}
@fragment
fn main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(0.,1.,0.,1.); }
            "#;

            let shader = crate::Shader::new(wgsl).expect("shader");
            let pass = crate::Pass::from_shader("mesh-explicit-indices", &shader);

            let mesh = Mesh::new();
            use crate::mesh::Vertex;
            // Four corners of a quad, sharing two positions across the
            // diagonal — dedup would collapse them; explicit indices keep
            // every corner so we can draw two triangles cleanly.
            mesh.add_vertices([
                Vertex::new([-0.5f32, -0.5]),
                Vertex::new([0.5f32, -0.5]),
                Vertex::new([0.5f32, 0.5]),
                Vertex::new([-0.5f32, 0.5]),
            ]);
            mesh.set_indices([0u32, 1, 2, 0, 2, 3]);

            pass.add_mesh(&mesh).expect("mesh is compatible");
            renderer.render(&pass, &target).expect("render ok");
        });
    }
}
