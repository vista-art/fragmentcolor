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

pub mod primitives;
pub use primitives::*;

pub(crate) mod builtins;

mod platform;

#[cfg(python)]
pub use platform::python::PyVertexValue;

use crate::{PassObject, Renderable, Shader};

#[derive(Clone, Debug)]
#[cfg_attr(python, pyclass)]
#[cfg_attr(wasm, wasm_bindgen)]
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
    #[lsp_doc("docs/api/geometry/mesh/set_instance_count.md")]
    pub fn set_instance_count(&self, n: u32) {
        *self.object.override_instances.write() = Some(n);
        self.object.invalidate_cache();
    }

    /// Clear the instance count override; fall back to instance buffer or 1.
    #[lsp_doc("docs/api/geometry/mesh/clear_instance_count.md")]
    pub fn clear_instance_count(&self) {
        *self.object.override_instances.write() = None;
        self.object.invalidate_cache();
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
                self.pass.add_shader(&shader);
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
        crate::pass::PassObject::ensure_flat_current(&self.pass);
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
            .map(|(k, val)| (k.clone(), PropBits::B(val.to_bytes())))
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

    // Schemas
    pub(crate) vertex_schema: RwLock<Option<VertexSchema>>, // derived from first vertex
    pub(crate) instance_schema: RwLock<Option<VertexSchema>>, // derived from first instance

    // Dirty flags
    dirty_vertices: RwLock<bool>,
    dirty_instances: RwLock<bool>,

    // Optional override for instance count (allows drawing without instance buffer)
    override_instances: RwLock<Option<u32>>,

    // GPU resources (created lazily)
    gpu: RwLock<Option<GpuStreams>>,

    // Cache for ensure_gpu results
    gpu_cache: RwLock<Option<(VertexBuffers, DrawCounts)>>,
    cache_valid: RwLock<bool>,
}

#[derive(Debug, Clone)]
pub(crate) struct VertexSchema {
    pub(crate) stride: u64,
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) fmt: wgpu::VertexFormat,
    pub(crate) size: u64,
}

#[derive(Debug)]
struct GpuStreams {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer_len: u32,
    instance_buffer: Option<(wgpu::Buffer, u32)>, // (buffer, count)
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
            vertex_schema: RwLock::new(None),
            instance_schema: RwLock::new(None),
            dirty_vertices: RwLock::new(false),
            dirty_instances: RwLock::new(false),
            override_instances: RwLock::new(None),
            gpu: RwLock::new(None),
            gpu_cache: RwLock::new(None),
            cache_valid: RwLock::new(false),
        }
    }

    fn add_vertex(&self, v: Vertex) {
        self.verts.write().push(v);
        *self.dirty_vertices.write() = true;
        self.invalidate_cache();
    }

    fn add_instance(&self, i: Instance) {
        self.insts.write().push(i);
        *self.dirty_instances.write() = true;
        self.invalidate_cache();
    }

    fn clear_instances(&self) {
        self.insts.write().clear();
        *self.dirty_instances.write() = true;
        self.invalidate_cache();
    }

    fn invalidate_cache(&self) {
        *self.cache_valid.write() = false;
        *self.gpu_cache.write() = None;
    }

    fn ensure_packed(&self) -> Result<(), MeshError> {
        // Derive schema from first vertex if missing
        if self.vertex_schema.read().is_none() {
            let v = match self.verts.read().first() {
                Some(v) => v.clone(),
                None => return Ok(()), // empty mesh; allowed
            };
            let s = derive_vertex_schema(&v)?;
            self.vertex_schema.write().replace(s);
        }
        if self.instance_schema.read().is_none()
            && let Some(first) = self.insts.read().first().cloned()
        {
            let s = derive_instance_schema(&first)?;
            self.instance_schema.write().replace(s);
        }

        // Pack vertices if dirty
        if *self.dirty_vertices.read() {
            let verts = self.verts.read();
            let schema = self.vertex_schema.read();
            // Dedup by full equality
            let mut map: HashMap<VertexKey, u32> = HashMap::new();
            let mut unique: Vec<&Vertex> = Vec::new();
            let mut idx: Vec<u32> = Vec::new();
            for v in verts.iter() {
                let key = VertexKey::from(v);
                if let Some(&i) = map.get(&key) {
                    idx.push(i);
                } else {
                    let i = unique.len() as u32;
                    map.insert(key, i);
                    unique.push(v);
                    idx.push(i);
                }
            }
            // Pack bytes
            let mut bytes = Vec::new();
            let _stride = schema.as_ref().map(|s| s.stride).unwrap_or(0);
            for v in unique.iter() {
                let Some(schema_ref) = schema.as_ref() else {
                    return Err(MeshError::NoVertexSchema);
                };
                pack_vertex(&mut bytes, v, schema_ref);
            }
            *self.packed_verts.write() = bytes;
            *self.indices.write() = idx;
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
                for ins in insts.iter() {
                    let Some(schema_ref) = schema.as_ref() else {
                        return Err(MeshError::NoInstanceSchema);
                    };
                    pack_instance(&mut bytes, ins, schema_ref)?;
                }
                *self.packed_insts.write() = bytes;
                *self.dirty_instances.write() = false;
            }
        }

        Ok(())
    }

    /// Get Vertex Buffers and Draw counts for this mesh,
    /// creating or updating GPU resources as needed.
    pub(crate) fn vertex_buffers(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(VertexBuffers, DrawCounts), MeshError> {
        if *self.cache_valid.read()
            && let Some(cached) = self.gpu_cache.read().as_ref()
        {
            return Ok(cached.clone());
        }

        let result = self.create_gpu_vertex_buffers(device, queue)?;

        *self.gpu_cache.write() = Some(result.clone());
        *self.cache_valid.write() = true;

        Ok(result)
    }

    fn create_gpu_vertex_buffers(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(VertexBuffers, DrawCounts), MeshError> {
        // Capture dirty flags before packing; ensure_packed() clears them
        let was_dirty_v = *self.dirty_vertices.read();
        let was_dirty_i = *self.dirty_instances.read();

        self.ensure_packed()?;
        let pv = self.packed_verts.read();
        let pi = self.packed_insts.read();
        let idx = self.indices.read();
        let si = self.instance_schema.read();

        // Create or grow buffers
        let mut gpu = self.gpu.write();

        // Decide whether we need to recreate GPU buffers
        let mut need_new = match gpu.as_ref() {
            None => true,
            Some(g) => g.instance_buffer_len as usize != idx.len() || was_dirty_v,
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
                (
                    Some((buf, {
                        let Some(schema_ref) = si.as_ref() else {
                            return Err(MeshError::NoInstanceSchema);
                        };
                        (pi.len() as u32) / (schema_ref.stride as u32)
                    })),
                    needed,
                )
            } else {
                (None, 0)
            };

            gpu.replace(GpuStreams {
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
                    if let Some((ref buf, _)) = g.instance_buffer {
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
            instance_buffer: g.instance_buffer.as_ref().map(|(b, _)| b.clone()),
        };

        let override_count = *self.override_instances.read();
        let draw_counts = DrawCounts {
            index_count: g.instance_buffer_len,
            instance_count: override_count
                .unwrap_or_else(|| g.instance_buffer.as_ref().map(|(_, c)| *c).unwrap_or(1)),
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
}

impl MeshObject {
    pub(crate) fn first_vertex_location_map(&self) -> (u32, HashMap<u32, String>) {
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
    pub(crate) fn first_instance_location_map(&self) -> HashMap<u32, String> {
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

fn derive_vertex_schema(vertex: &Vertex) -> Result<VertexSchema, MeshError> {
    let mut fields: Vec<Field> = Vec::new();
    // position first; single key with dynamic format
    match vertex.dimensions {
        0..=2 => fields.push(Field {
            name: "position".into(),
            fmt: wgpu::VertexFormat::Float32x2,
            size: 8,
        }),
        _ => fields.push(Field {
            name: "position".into(),
            fmt: wgpu::VertexFormat::Float32x3,
            size: 12,
        }),
    }
    // remaining properties in sorted order
    let mut rest: Vec<(&String, &VertexValue)> = vertex.properties.iter().collect();
    rest.sort_by(|a, b| a.0.cmp(b.0));
    for (k, val) in rest {
        fields.push(Field {
            name: k.clone(),
            fmt: val.format(),
            size: val.size(),
        });
    }
    let stride = fields.iter().map(|f| f.size).sum();
    Ok(VertexSchema { stride, fields })
}

fn derive_instance_schema(i: &Instance) -> Result<VertexSchema, MeshError> {
    let mut fields: Vec<Field> = Vec::new();
    // Only explicit per-instance properties; sorted by key
    let mut rest: Vec<(&String, &VertexValue)> = i.properties.iter().collect();
    rest.sort_by(|a, b| a.0.cmp(b.0));
    for (k, val) in rest {
        fields.push(Field {
            name: k.clone(),
            fmt: val.format(),
            size: val.size(),
        });
    }
    let stride = fields.iter().map(|f| f.size).sum();
    Ok(VertexSchema { stride, fields })
}

fn pack_vertex(out: &mut Vec<u8>, v: &Vertex, schema: &VertexSchema) {
    for f in schema.fields.iter() {
        match f.name.as_str() {
            "position" => {
                if matches!(v.dimensions, 0..=2) && f.fmt == wgpu::VertexFormat::Float32x2 {
                    let p = [v.position.0.x, v.position.0.y];
                    out.extend_from_slice(bytemuck::cast_slice(&p));
                } else if v.dimensions >= 3 && f.fmt == wgpu::VertexFormat::Float32x3 {
                    let p = [v.position.0.x, v.position.0.y, v.position.0.z];
                    out.extend_from_slice(bytemuck::cast_slice(&p));
                } else {
                    // format mismatch or missing comps; zero-fill
                    out.extend(std::iter::repeat_n(0u8, f.size as usize));
                }
            }
            name => {
                if let Some(val) = v.properties.get(name) {
                    out.extend_from_slice(&val.to_bytes());
                } else {
                    // zero-fill absent optional properties
                    out.extend(std::iter::repeat_n(0u8, f.size as usize));
                }
            }
        }
    }
}

fn pack_instance(out: &mut Vec<u8>, i: &Instance, schema: &VertexSchema) -> Result<(), MeshError> {
    for f in schema.fields.iter() {
        if let Some(val) = i.properties.get(&f.name) {
            out.extend_from_slice(&val.to_bytes());
        } else {
            out.extend(std::iter::repeat_n(0u8, f.size as usize));
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
        let schema = derive_vertex_schema(&v).expect("schema");
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
            use crate::mesh::Vertex;
            mesh.add_vertices([
                Vertex::new([-0.5f32, -0.5f32]),
                Vertex::new([0.5f32, -0.5f32]),
                Vertex::new([0.0f32, 0.5f32]),
            ]);
            let inst = Vertex::new([0.0f32, 0.0]).set("id", 7u32).create_instance();
            mesh.add_instances([inst.clone(), inst]);

            // Build GPU buffers
            let instance = crate::renderer::platform::all::create_instance().await;
            let adapter = crate::renderer::platform::all::request_adapter(&instance, None)
                .await
                .expect("adapter");
            let (device, queue) = crate::renderer::platform::all::request_device(&adapter)
                .await
                .expect("device");

            // Without override, instance_count == 2
            let (_bufs, counts) = mesh.object.vertex_buffers(&device, &queue).expect("vb");
            assert_eq!(counts.instance_count, 2);

            // With override, instance_count == 5
            mesh.set_instance_count(5);
            let (_bufs2, counts2) = mesh.object.vertex_buffers(&device, &queue).expect("vb2");
            assert_eq!(counts2.instance_count, 5);

            // Clear override, back to 2
            mesh.clear_instance_count();
            let (_bufs3, counts3) = mesh.object.vertex_buffers(&device, &queue).expect("vb3");
            assert_eq!(counts3.instance_count, 2);
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
}
