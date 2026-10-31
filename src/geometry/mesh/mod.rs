//! `Mesh`, and `MeshObject`, the vertices, instances and indices its clones share.

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

use super::{Instance, MeshError, Vertex};

use crate::RendererError;
use crate::core::VertexLayouts;
use crate::{PassObject, Renderable, Shader, ShaderError};

mod gpu_buffers;
use gpu_buffers::{Attached, Cached, GpuStreams};
pub(crate) use gpu_buffers::{DrawCounts, Upload, VertexBuffers};
mod packing;
pub(crate) use packing::Schemas;
use packing::{
    Packed, Repacked, check_indices, check_layout, check_vertices, count, pack_instance,
    pack_vertex,
};
mod schema;
pub(crate) use schema::{AttributeTypes, Binding, VertexSchema, build_columns};
use schema::{check_conflicts, derive_instance_schema, derive_vertex_schema, reset_schema};

#[derive(Clone, Debug)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[lsp_doc("docs/api/geometry/mesh/mesh.md")]
pub struct Mesh {
    pub(crate) object: Arc<MeshObject>,
    pub(crate) pass: Arc<crate::core::PassObject>,
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
        let object = Arc::new(MeshObject::new());
        let pass = crate::core::PassObject::for_mesh("Mesh Internal Pass", object.clone());
        Self {
            object,
            pass: Arc::new(pass),
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
        self.object.add_vertices(vec![v.into()]);
    }

    #[lsp_doc("docs/api/geometry/mesh/add_vertices.md")]
    pub fn add_vertices<I, V>(&self, vertices: I)
    where
        I: IntoIterator<Item = V>,
        V: Into<Vertex>,
    {
        self.object
            .add_vertices(vertices.into_iter().map(Into::into).collect());
    }

    #[lsp_doc("docs/api/geometry/mesh/add_instance.md")]
    pub fn add_instance<T: Into<Instance>>(&self, instance_buffer: T) {
        self.object.add_instances(vec![instance_buffer.into()]);
    }

    #[lsp_doc("docs/api/geometry/mesh/add_instances.md")]
    pub fn add_instances<I, T>(&self, instances: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<Instance>,
    {
        self.object
            .add_instances(instances.into_iter().map(Into::into).collect());
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
/// That shader fixes the types it gives plain numbers, and it records no reads.
/// The renderer builds it and checks the mesh on every render that reaches its pass,
/// a `Pass::require` of the mesh included, and a mesh it refuses makes `render` say why.
impl Renderable for Mesh {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        self.pass.ensure_preview();
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
    /// Counts the changes to the mesh. A cached upload from an older one is stale.
    generation: AtomicU64,
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

    /// Adds `vertices` as one change, under one hold of the list, so a render
    /// on another thread sees all of them or none. An empty list changes nothing.
    fn add_vertices(&self, vertices: Vec<Vertex>) {
        if vertices.is_empty() {
            return;
        }
        // Grow the local-space AABB before stashing the vertices. 2D positions
        // contribute z = 0, which is the same value the vertex shader sees
        // (the `position` slot zero-pads the missing components).
        let mut minimum = self.aabb_min.write();
        let mut maximum = self.aabb_max.write();
        for vertex in vertices.iter() {
            let [x, y, z, _] = vertex.position.0.to_array();
            let point = glam::Vec3::new(x, y, z);
            *minimum = minimum.min(point);
            *maximum = maximum.max(point);
        }
        drop(minimum);
        drop(maximum);
        let mut verts = self.verts.write();
        verts.extend(vertices);
        #[cfg(test)]
        crate::testing::race_hooks::run("pushed");
        self.touch_vertices();
        drop(verts);
        #[cfg(test)]
        crate::testing::race_hooks::run("added");
    }

    /// Local-space AABB of every Vertex added so far. Returns the sentinel
    /// pair (`+inf`, `-inf`) for an empty mesh; callers detect this and
    /// fall back to a centroid of zero. Internal-only — glam types stay
    /// inside the crate per the no-glam-on-public-API rule.
    pub(crate) fn aabb_local(&self) -> (glam::Vec3, glam::Vec3) {
        (*self.aabb_min.read(), *self.aabb_max.read())
    }

    /// Adds `instances` as one change, under one hold of the list, so a render
    /// on another thread sees all of them or none. An empty list changes nothing.
    fn add_instances(&self, instances: Vec<Instance>) {
        if instances.is_empty() {
            return;
        }
        let mut insts = self.insts.write();
        insts.extend(instances);
        // Adding a real instance invalidates any prior set_instance_count override
        // (otherwise the GPU would draw past the actual buffer end).
        *self.override_instances.write() = None;
        #[cfg(test)]
        crate::testing::race_hooks::run("instance pushed");
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
        #[cfg(test)]
        crate::testing::race_hooks::run("instances cleared");
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

    /// Marks every cached upload stale. It takes no lock,
    /// so a mutator may call it whatever it holds. Kept layouts follow the pack's schemas.
    pub(crate) fn invalidate_cache(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
    }

    /// The count of changes to the mesh, which moves on every one.
    pub(crate) fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    /// Builds the quick view's shader on `pass` from the first vertex, once there is one,
    /// and checks the mesh against it. An empty mesh builds nothing and draws nothing.
    pub(crate) fn preview_in(self: &Arc<Self>, pass: &PassObject) {
        if self.verts.read().is_empty() {
            return;
        }
        let shader = pass.ensure_shader(|| {
            // Copied once, so no vertex guard is held while the shader compiles or checks.
            let first = self.verts.read().first().cloned();
            match first.as_ref().map(Shader::from_vertex) {
                Some(Ok(shader)) => shader.object,
                _ => Shader::default().object,
            }
        });
        shader.preview(self);
    }

    /// The layouts `build` makes for the attached shader `shader` from the pack whose
    /// `schemas` an upload carries. They stay until another pack's schemas arrive or the
    /// shader lets the mesh go, so two Renderers that packed different data never share them.
    /// A shader that doesn't hold the mesh keeps none.
    pub(crate) fn fetch_layouts(
        &self,
        shader: u64,
        schemas: &Arc<Schemas>,
        build: impl FnOnce() -> Result<VertexLayouts, RendererError>,
    ) -> Result<VertexLayouts, RendererError> {
        if let Some(attached) = self.required.read().get(&shader)
            && let Some((kept, layouts)) = attached.layouts.as_ref()
            && Arc::ptr_eq(kept, schemas)
        {
            return Ok(layouts.clone());
        }
        let layouts = build()?;
        if let Some(attached) = self.required.write().get_mut(&shader) {
            attached.layouts = Some((schemas.clone(), layouts.clone()));
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

    /// Records what a shader's attach found.
    ///
    /// Untyped attributes take its types, which repack what they change.
    /// Vertices and instances added later must hold the attributes it reads.
    /// A binding that assumed the Model columns sets them as the instance layout
    /// before it records the reads, unless an instance or a layout arrived since.
    /// A type another shader gave one of them meanwhile fails, and it records nothing.
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
        crate::testing::race_hooks::run("columns");
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
        *schema = Some(build_columns());
        true
    }

    /// Drops what the shader `shader` reads and the layouts it kept,
    /// once that shader no longer holds the mesh.
    pub(crate) fn detach(&self, shader: u64) {
        self.required.write().remove(&shader);
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
        crate::testing::race_hooks::run("pack");
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
        packed.vertices = bytes;
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
        packed.instances = bytes;
        packed.rows = insts.len();
        Arc::make_mut(&mut packed.schemas).instance = schema.clone();
        Ok(())
    }
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

#[cfg(test)]
mod tests;
