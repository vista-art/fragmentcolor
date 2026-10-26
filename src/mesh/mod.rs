use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg(python)]
use pyo3::prelude::*;

pub mod error;
pub use error::*;

pub mod vertex;
pub use vertex::*;

mod data;
pub use data::VertexData;
pub(crate) use data::{Shape, describe};

pub mod primitives;
pub use primitives::*;

pub(crate) mod builtins;

mod platform;

use crate::renderer::RenderContext;
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
impl Renderable for Mesh {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        if self.pass.shaders.read().is_empty() {
            if let Some(first) = self.object.verts.read().first().cloned() {
                let shader = Shader::from_vertex(&first);
                self.pass.push_shader(shader.object.clone());
                _ = shader.add_mesh(self);
            }
        } else if let Some(shader) = self.pass.shaders.read().last().cloned() {
            let is_attached = shader
                .meshes
                .read()
                .iter()
                .any(|mesh| Arc::ptr_eq(mesh, &self.object));

            if !is_attached {
                _ = shader.add_mesh(self.object.clone());
            }
        }
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

    // Derived, packed bytes
    pub(crate) packed_verts: RwLock<Vec<u8>>, // unique verts packed by schema
    pub(crate) packed_insts: RwLock<Vec<u8>>, // instances packed by schema

    pub(crate) indices: RwLock<Vec<u32>>, // indices referencing unique verts
    pub(crate) indices_overridden: RwLock<bool>, // user supplied indices via set_indices

    // Schemas
    pub(crate) vertex_schema: RwLock<Option<VertexSchema>>, // derived from first vertex
    pub(crate) instance_schema: RwLock<Option<VertexSchema>>, // derived from first instance

    /// The format the first shader to read each untyped attribute gave it.
    /// A leaf lock: never take another lock while holding this one.
    pub(crate) types: RwLock<AttributeTypes>,

    // Dirty flags
    dirty_vertices: RwLock<bool>,
    dirty_instances: RwLock<bool>,

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
    gpu: RwLock<Option<GpuStreams>>,

    /// The last `vertex_buffers` result, for the device `gpu` lives on.
    gpu_cache: RwLock<Option<(VertexBuffers, DrawCounts)>>,
    cache_valid: RwLock<bool>,
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

/// A mesh's buffers on the device of one `Renderer`. Another `Renderer`
/// that draws the mesh uploads buffers of its own in their place.
#[derive(Debug)]
struct GpuStreams {
    /// The `RenderContext::id` of the context whose device holds the buffers.
    owner: u64,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer_len: u32,
    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_capacity: u64,
}

impl MeshObject {
    fn new() -> Self {
        Self {
            verts: RwLock::new(Vec::new()),
            insts: RwLock::new(Vec::new()),
            packed_verts: RwLock::new(Vec::new()),
            packed_insts: RwLock::new(Vec::new()),
            indices: RwLock::new(Vec::new()),
            indices_overridden: RwLock::new(false),
            vertex_schema: RwLock::new(None),
            instance_schema: RwLock::new(None),
            types: RwLock::new(AttributeTypes::default()),
            dirty_vertices: RwLock::new(false),
            dirty_instances: RwLock::new(false),
            override_instances: RwLock::new(None),
            aabb_min: RwLock::new(glam::Vec3::splat(f32::INFINITY)),
            aabb_max: RwLock::new(glam::Vec3::splat(f32::NEG_INFINITY)),
            gpu: RwLock::new(None),
            gpu_cache: RwLock::new(None),
            cache_valid: RwLock::new(false),
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
        self.verts.write().push(v);
        *self.dirty_vertices.write() = true;
        self.invalidate_cache();
    }

    /// Local-space AABB of every Vertex added so far. Returns the sentinel
    /// pair (`+inf`, `-inf`) for an empty mesh; callers detect this and
    /// fall back to a centroid of zero. Internal-only — glam types stay
    /// inside the crate per the no-glam-on-public-API rule.
    pub(crate) fn aabb_local(&self) -> (glam::Vec3, glam::Vec3) {
        (*self.aabb_min.read(), *self.aabb_max.read())
    }

    fn add_instance(&self, i: Instance) {
        self.insts.write().push(i);
        *self.dirty_instances.write() = true;
        // Adding a real instance invalidates any prior set_instance_count override
        // (otherwise the GPU would draw past the actual buffer end).
        *self.override_instances.write() = None;
        self.invalidate_cache();
    }

    fn clear_instances(&self) {
        self.insts.write().clear();
        // The next instance sets the layout again, but the model columns stay.
        let mut schema = self.instance_schema.write();
        if schema
            .as_ref()
            .is_some_and(|schema| *schema != model_schema())
        {
            *schema = None;
        }
        drop(schema);
        *self.dirty_instances.write() = true;
        // Reset to default state: no buffer, no count override, draws as a single instance.
        *self.override_instances.write() = None;
        self.invalidate_cache();
    }

    fn set_indices(&self, indices: Vec<u32>) {
        *self.indices.write() = indices;
        *self.indices_overridden.write() = true;
        // Vertices need to be re-packed in insertion order (no dedup).
        *self.dirty_vertices.write() = true;
        self.invalidate_cache();
    }

    fn clear_indices(&self) {
        *self.indices_overridden.write() = false;
        // Force ensure_packed to re-derive indices via the dedup path.
        *self.dirty_vertices.write() = true;
        self.invalidate_cache();
    }

    pub(crate) fn invalidate_cache(&self) {
        *self.cache_valid.write() = false;
        *self.gpu_cache.write() = None;
    }

    /// Declare the per-Model instance vertex schema (four `vec4<f32>` columns
    /// of a `mat4x4<f32>` at attribute names `model_0..model_3`) without
    /// adding any instance data. `Pass::add_model` calls this on first use of
    /// a Mesh so the pipeline created from this Mesh's Shader has a slot-1
    /// instance VertexBufferLayout matching the PBR vertex shader's
    /// `@location(3..6)` inputs. Idempotent: a no-op if the Mesh already has
    /// an instance schema (set either by the user via `add_instance` or by a
    /// prior `Pass::add_model` call).
    pub(crate) fn declare_model_instance_schema(&self) {
        if self.instance_schema.read().is_some() {
            return;
        }
        *self.instance_schema.write() = Some(model_schema());
        self.invalidate_cache();
    }

    /// Whether only Models give this mesh instances:
    /// it declares the Model columns and holds no instance of its own.
    pub(crate) fn awaits_models(&self) -> bool {
        self.insts.read().is_empty()
            && self.instance_schema.read().as_ref() == Some(&model_schema())
    }

    /// Records the types a shader gave untyped attributes and repacks what they change.
    /// Records nothing when another shader has typed one of them another way meanwhile.
    pub(crate) fn declare(&self, added: AttributeTypes) -> Result<(), ShaderError> {
        if added.is_empty() {
            return Ok(());
        }
        {
            let mut types = self.types.write();
            check_conflicts(&types.vertex, &added.vertex)?;
            check_conflicts(&types.instance, &added.instance)?;
            types.vertex.extend(added.vertex.clone());
            types.instance.extend(added.instance.clone());
        }
        if !added.vertex.is_empty() {
            reset_schema(&self.vertex_schema, &added.vertex);
            *self.dirty_vertices.write() = true;
        }
        if !added.instance.is_empty() {
            reset_schema(&self.instance_schema, &added.instance);
            *self.dirty_instances.write() = true;
        }
        self.invalidate_cache();
        Ok(())
    }

    fn ensure_packed(&self) -> Result<(), MeshError> {
        // Derive and store each missing schema under its own lock, so a `declare`
        // either lands before the derive reads `types` or resets the stored schema.
        {
            let verts = self.verts.read();
            let Some(first) = verts.first() else {
                return Ok(()); // empty mesh; allowed
            };
            let mut schema = self.vertex_schema.write();
            if schema.is_none() {
                *schema = Some(derive_vertex_schema(first, &self.types.read().vertex));
            }
        }
        {
            let insts = self.insts.read();
            let mut schema = self.instance_schema.write();
            if schema.is_none()
                && let Some(first) = insts.first()
            {
                *schema = Some(derive_instance_schema(first, &self.types.read().instance));
            }
        }

        // Pack vertices if dirty
        if *self.dirty_vertices.read() {
            let verts = self.verts.read();
            let schema = self.vertex_schema.read();
            let overridden = *self.indices_overridden.read();
            let mut bytes = Vec::new();
            if overridden {
                // Pack every vertex in insertion order; preserve user-supplied indices.
                for (index, v) in verts.iter().enumerate() {
                    let Some(schema_ref) = schema.as_ref() else {
                        return Err(MeshError::NoVertexSchema);
                    };
                    pack_vertex(&mut bytes, index, v, schema_ref)?;
                }
            } else {
                // Dedup by full equality and derive indices.
                let mut map: HashMap<VertexKey, u32> = HashMap::new();
                let mut unique: Vec<(usize, &Vertex)> = Vec::new();
                let mut idx: Vec<u32> = Vec::new();
                for (index, v) in verts.iter().enumerate() {
                    let key = VertexKey::from(v);
                    if let Some(&i) = map.get(&key) {
                        idx.push(i);
                    } else {
                        let i = unique.len() as u32;
                        map.insert(key, i);
                        unique.push((index, v));
                        idx.push(i);
                    }
                }
                for (index, v) in unique {
                    let Some(schema_ref) = schema.as_ref() else {
                        return Err(MeshError::NoVertexSchema);
                    };
                    pack_vertex(&mut bytes, index, v, schema_ref)?;
                }
                *self.indices.write() = idx;
            }
            *self.packed_verts.write() = bytes;
            *self.dirty_vertices.write() = false;
        }
        // Pack instances if dirty
        if *self.dirty_instances.read() {
            let insts = self.insts.read();
            let schema = self.instance_schema.read();
            if insts.is_empty() {
                *self.packed_insts.write() = Vec::new();
                *self.dirty_instances.write() = false;
            } else {
                let mut bytes = Vec::new();
                for (index, ins) in insts.iter().enumerate() {
                    let Some(schema_ref) = schema.as_ref() else {
                        return Err(MeshError::NoInstanceSchema);
                    };
                    pack_instance(&mut bytes, index, ins, schema_ref)?;
                }
                *self.packed_insts.write() = bytes;
                *self.dirty_instances.write() = false;
            }
        }

        Ok(())
    }

    /// Get Vertex Buffers and Draw counts for this mesh,
    /// creating or updating GPU resources on the device of `context` as needed.
    /// Buffers on another context's device give way to new ones.
    pub(crate) fn vertex_buffers(
        &self,
        context: &RenderContext,
    ) -> Result<(VertexBuffers, DrawCounts), MeshError> {
        let owned = self
            .gpu
            .read()
            .as_ref()
            .is_some_and(|gpu| gpu.owner == context.id);
        if owned
            && *self.cache_valid.read()
            && let Some(cached) = self.gpu_cache.read().as_ref()
        {
            return Ok(cached.clone());
        }

        let result = self.upload_vertex_buffers(context)?;

        *self.gpu_cache.write() = Some(result.clone());
        *self.cache_valid.write() = true;

        Ok(result)
    }

    fn upload_vertex_buffers(
        &self,
        context: &RenderContext,
    ) -> Result<(VertexBuffers, DrawCounts), MeshError> {
        let (device, queue) = (&context.device, &context.queue);
        // Capture dirty flags before packing; ensure_packed() clears them
        let was_dirty_v = *self.dirty_vertices.read();
        let was_dirty_i = *self.dirty_instances.read();

        self.ensure_packed()?;
        let pv = self.packed_verts.read();
        let pi = self.packed_insts.read();
        let idx = self.indices.read();
        let si = self.instance_schema.read();

        // Count rows on every upload, so a list that shrinks into the old buffer draws its own length.
        let instance_rows = if pi.is_empty() {
            None
        } else {
            let Some(schema) = si.as_ref() else {
                return Err(MeshError::NoInstanceSchema);
            };
            Some((pi.len() as u64 / schema.stride) as u32)
        };

        // Create or grow buffers
        let mut gpu = self.gpu.write();

        // Decide whether we need to recreate GPU buffers. Buffers on
        // another device can't draw here, so they give way to new ones.
        let mut need_new = match gpu.as_ref() {
            None => true,
            Some(g) => {
                g.owner != context.id || g.instance_buffer_len as usize != idx.len() || was_dirty_v
            }
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
            queue.write_buffer(&vertex_buffer, 0, &pv);
            queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&idx));

            let (instance_buffer, instance_bytes) = if !pi.is_empty() {
                let needed = pi.len() as u64;
                let buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Instance Buffer"),
                    size: needed,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&buf, 0, &pi);
                (Some(buf), needed)
            } else {
                (None, 0)
            };

            gpu.replace(GpuStreams {
                owner: context.id,
                vertex_buffer,
                index_buffer,
                instance_buffer_len: idx.len() as u32,
                instance_buffer,
                instance_buffer_capacity: instance_bytes,
            });
        } else {
            // Update contents if needed without recreating buffers
            if let Some(g) = gpu.as_ref() {
                if was_dirty_v {
                    queue.write_buffer(&g.vertex_buffer, 0, &pv);
                    *self.dirty_vertices.write() = false;
                }
                if was_dirty_i {
                    if let Some(buf) = &g.instance_buffer {
                        // Safe to write since capacity check passed above
                        queue.write_buffer(buf, 0, &pi);
                    }
                    *self.dirty_instances.write() = false;
                }
            }
        }

        let Some(g) = gpu.as_ref() else {
            return Err(MeshError::NoGpuStreams);
        };
        let vertex_buffers = VertexBuffers {
            vertex_buffer: g.vertex_buffer.clone(),
            index_buffer: g.index_buffer.clone(),
            instance_buffer: g.instance_buffer.clone(),
        };

        let override_count = *self.override_instances.read();
        let draw_counts = DrawCounts {
            index_count: g.instance_buffer_len,
            instance_count: override_count.or(instance_rows).unwrap_or(1),
            instance_rows,
        };

        Ok((vertex_buffers, draw_counts))
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

/// The instance layout of a `Model`: the four `vec4<f32>` columns of its transform.
fn model_schema() -> VertexSchema {
    let fields = ["model_0", "model_1", "model_2", "model_3"]
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
    let mut fields = vec![Field::new("position", position)];
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

fn pack_vertex(
    out: &mut Vec<u8>,
    index: usize,
    v: &Vertex,
    schema: &VertexSchema,
) -> Result<(), MeshError> {
    for f in schema.fields.iter() {
        match f.name.as_str() {
            "position" => {
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
                // zero-fill absent optional properties
                None => out.extend(std::iter::repeat_n(0u8, f.size as usize)),
            },
        }
    }
    Ok(())
}

fn pack_instance(
    out: &mut Vec<u8>,
    index: usize,
    i: &Instance,
    schema: &VertexSchema,
) -> Result<(), MeshError> {
    for f in schema.fields.iter() {
        match i.properties.get(&f.name) {
            Some(value) => value.write(f.fmt, out).map_err(|misfit| {
                MeshError::InvalidInstance(misfit.explain(&f.name, &format!("instance {index}")))
            })?,
            None => out.extend(std::iter::repeat_n(0u8, f.size as usize)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        ]);
        match mesh.object.ensure_packed() {
            Err(MeshError::InvalidVertex(text)) => {
                assert_eq!(text, "uv: expected vec3<f32>, got vec2<f32> at vertex 1");
            }
            other => panic!("expected an invalid vertex, got {other:?}"),
        }

        let mesh = Mesh::new();
        mesh.add_vertices([Vertex::new([0.0f32, 0.0, 0.0]), Vertex::new([1.0f32, 0.0])]);
        assert!(matches!(
            mesh.object.ensure_packed(),
            Err(MeshError::LayoutMismatch(_))
        ));

        let mesh = Mesh::new();
        mesh.add_vertex([0.0f32, 0.0]);
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

    // Story: a plain number packs as a float until a shader types it, then
    // the mesh repacks it in that type and keeps it for the mesh's lifetime.
    #[test]
    fn untyped_attributes_pack_as_floats_until_declared() {
        let mesh = Mesh::new();
        mesh.add_vertices([
            Vertex::new([0.0f32, 0.0]).set("id", 3),
            Vertex::new([1.0f32, 0.0]).set("id", 4),
        ]);
        let field = |mesh: &Mesh| -> (wgpu::VertexFormat, Vec<u8>, Vec<u8>) {
            mesh.object.ensure_packed().expect("pack");
            let schema = mesh.object.vertex_schema.read().clone().expect("schema");
            let bytes = mesh.object.packed_verts.read().clone();
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
        mesh.object.declare(added).expect("declare");
        assert!(mesh.object.vertex_schema.read().is_none());
        assert!(*mesh.object.dirty_vertices.read());

        let (format, first, second) = field(&mesh);
        assert_eq!(format, wgpu::VertexFormat::Uint32);
        assert_eq!(first, 3u32.to_ne_bytes());
        assert_eq!(second, 4u32.to_ne_bytes());

        let mut conflict = AttributeTypes::default();
        conflict
            .vertex
            .insert("id".to_string(), wgpu::VertexFormat::Float32);
        let error = mesh.object.declare(conflict).expect_err("conflict");
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
            mesh.object.vertex_buffers(&context).expect("buffers");
            assert!(*mesh.object.cache_valid.read());

            let second = crate::Shader::new(wgsl).expect("shader");
            second.add_mesh(&mesh).expect("second attach");
            first.validate_mesh(&mesh).expect("validate again");
            assert!(*mesh.object.cache_valid.read());
            assert!(!*mesh.object.dirty_vertices.read());
        });
    }

    #[test]
    fn ensure_packed_deduplicates_and_indices_track() {
        let mesh = Mesh::new();
        use crate::mesh::Vertex;
        let a = Vertex::new([0.0f32, 0.0]).set("uv", [0.0f32, 0.0]);
        let b = Vertex::new([1.0f32, 0.0]).set("uv", [1.0f32, 0.0]);
        let c = Vertex::new([0.0f32, 1.0]).set("uv", [0.0f32, 1.0]);
        mesh.add_vertices([a.clone(), b.clone(), a.clone(), c.clone()]);

        // Force packing and inspect internals
        mesh.object.ensure_packed().expect("pack");
        let stride = mesh.object.vertex_schema.read().as_ref().unwrap().stride;
        let pv = mesh.object.packed_verts.read().clone();
        let idx = mesh.object.indices.read().clone();
        // Unique verts should be a, b, c => 3 * stride bytes
        assert_eq!(pv.len() as u64, 3 * stride);
        // Indices mirror [a,b,a,c] => [0,1,0,2]
        assert_eq!(idx, vec![0, 1, 0, 2]);
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
            let (_bufs, counts) = mesh.object.vertex_buffers(&context).expect("vb");
            assert_eq!(counts.instance_count, 2);

            // With override, instance_count == 5
            mesh.set_instance_count(5);
            let (_bufs2, counts2) = mesh.object.vertex_buffers(&context).expect("vb2");
            assert_eq!(counts2.instance_count, 5);

            // Adding an instance auto-clears the override (otherwise the GPU would
            // draw past the actual buffer end and read garbage). count falls back
            // to the new buffer length (3).
            mesh.add_instance(Instance::new().set("id", 8u32));
            let (_bufs3, counts3) = mesh.object.vertex_buffers(&context).expect("vb3");
            assert_eq!(counts3.instance_count, 3);

            // clear_instances resets everything: empty buffer + no override → 1.
            mesh.set_instance_count(99);
            mesh.clear_instances();
            let (_bufs4, counts4) = mesh.object.vertex_buffers(&context).expect("vb4");
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
        let pv = mesh.object.packed_verts.read().clone();
        let idx = mesh.object.indices.read().clone();
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
        assert_eq!(mesh.object.packed_verts.read().len() as u64, 3 * stride);
        assert_eq!(*mesh.object.indices.read(), vec![0, 1, 2]);

        // Clear and re-pack: dedup should collapse `a` and produce derived indices.
        mesh.clear_indices();
        mesh.object.ensure_packed().expect("pack auto");
        assert_eq!(mesh.object.packed_verts.read().len() as u64, 2 * stride);
        assert_eq!(*mesh.object.indices.read(), vec![0, 1, 0]);
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
