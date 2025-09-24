use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::collections::HashMap;
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

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass)]
#[derive(Clone, Debug)]
#[lsp_doc("docs/api/core/mesh/mesh.md")]
pub struct Mesh {
    pub(crate) object: Arc<MeshObject>,
    pub(crate) pass: Arc<crate::pass::PassObject>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh {
    #[lsp_doc("docs/api/core/mesh/new.md")]
    pub fn new() -> Self {
        Self {
            object: Arc::new(MeshObject::new()),
            pass: Arc::new(crate::pass::PassObject::new(
                "Mesh Debug Pass",
                crate::pass::PassType::Render,
            )),
        }
    }

    #[lsp_doc("docs/api/core/mesh/from_vertices.md")]
    pub fn from_vertices<I>(verts: I) -> Self
    where
        I: IntoIterator<Item = Vertex>,
    {
        let mut m = Mesh::new();
        m.add_vertices(verts);
        m
    }

    #[lsp_doc("docs/api/core/mesh/add_vertex.md")]
    pub fn add_vertex<V: Into<Vertex>>(&mut self, v: V) {
        self.object.add_vertex_internal(v.into());
    }

    #[lsp_doc("docs/api/core/mesh/add_vertices.md")]
    pub fn add_vertices<I>(&mut self, verts: I)
    where
        I: IntoIterator<Item = Vertex>,
    {
        for v in verts {
            self.object.add_vertex_internal(v);
        }
    }

    #[lsp_doc("docs/api/core/mesh/add_instance.md")]
    pub fn add_instance<T: Into<Instance>>(&mut self, instance_buffer: T) {
        self.object.add_instance_internal(instance_buffer.into());
    }

    #[lsp_doc("docs/api/core/mesh/add_instances.md")]
    pub fn add_instances<I, T>(&mut self, list: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<Instance>,
    {
        for it in list {
            self.object.add_instance_internal(it.into());
        }
    }

    #[lsp_doc("docs/api/core/mesh/clear_instances.md")]
    pub fn clear_instances(&mut self) {
        self.object.clear_instances_internal();
    }

    /// Override how many instances to draw (when not using per-instance attributes).
    #[lsp_doc("docs/api/core/mesh/set_instance_count.md")]
    pub fn set_instance_count(&mut self, n: u32) {
        *self.object.override_instances.write() = Some(n);
    }

    /// Clear the instance count override; fall back to instance buffer or 1.
    #[lsp_doc("docs/api/core/mesh/clear_instance_count.md")]
    pub fn clear_instance_count(&mut self) {
        *self.object.override_instances.write() = None;
    }
}

// -----------------------------
// Renderable impl ( for Mesh quick-view)
// -----------------------------
impl crate::renderer::Renderable for Mesh {
    fn passes(&self) -> impl IntoIterator<Item = &crate::pass::PassObject> {
        // If this internal pass has no shader yet, build one from the first vertex
        if self.pass.shaders.read().is_empty() {
            if let Some(first) = self.object.verts.read().first().cloned()
                && let Ok(shader) = crate::Shader::from_vertex(&first)
            {
                self.pass.add_shader(&shader);
                shader.add_mesh(self);
            }
        } else if let Some(sh) = self.pass.shaders.read().last().cloned() {
            let attached = sh
                .meshes
                .read()
                .iter()
                .any(|m| Arc::ptr_eq(m, &self.object));
            if !attached {
                sh.add_mesh_internal(self.object.clone());
            }
        }
        vec![self.pass.as_ref()]
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
    pub(crate) schema_v: RwLock<Option<Schema>>, // derived from first vertex
    pub(crate) schema_i: RwLock<Option<Schema>>, // derived from first instance

    // Dirty flags
    dirty_v: RwLock<bool>,
    dirty_i: RwLock<bool>,

    // Optional override for instance count (allows drawing without instance buffer)
    override_instances: RwLock<Option<u32>>,

    // GPU resources (created lazily)
    gpu: RwLock<Option<GpuStreams>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Schema {
    pub(crate) stride: u64,
    // ordered fields
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
    // Capacity in bytes of the current instance buffer; 0 when None
    instance_bytes: u64,
}

impl MeshObject {
    fn new() -> Self {
        Self {
            verts: RwLock::new(Vec::new()),
            insts: RwLock::new(Vec::new()),
            packed_verts: RwLock::new(Vec::new()),
            packed_insts: RwLock::new(Vec::new()),
            indices: RwLock::new(Vec::new()),
            schema_v: RwLock::new(None),
            schema_i: RwLock::new(None),
            dirty_v: RwLock::new(false),
            dirty_i: RwLock::new(false),
            override_instances: RwLock::new(None),
            gpu: RwLock::new(None),
        }
    }

    fn add_vertex_internal(&self, v: Vertex) {
        self.verts.write().push(v);
        *self.dirty_v.write() = true;
    }
    fn add_instance_internal(&self, i: Instance) {
        self.insts.write().push(i);
        *self.dirty_i.write() = true;
    }
    fn clear_instances_internal(&self) {
        self.insts.write().clear();
        *self.dirty_i.write() = true;
    }

    fn ensure_packed(&self) -> Result<(), MeshError> {
        // Derive schema from first vertex if missing
        if self.schema_v.read().is_none() {
            let v = match self.verts.read().first() {
                Some(v) => v.clone(),
                None => return Ok(()), // empty mesh; allowed
            };
            let s = derive_vertex_schema(&v)?;
            self.schema_v.write().replace(s);
        }
        if self.schema_i.read().is_none()
            && let Some(first) = self.insts.read().first().cloned()
        {
            let s = derive_instance_schema(&first)?;
            self.schema_i.write().replace(s);
        }

        // Pack vertices if dirty
        if *self.dirty_v.read() {
            let verts = self.verts.read();
            let schema = self.schema_v.read();
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
                pack_vertex(&mut bytes, v, schema.as_ref().unwrap());
            }
            *self.packed_verts.write() = bytes;
            *self.indices.write() = idx;
            *self.dirty_v.write() = false;
        }
        // Pack instances if dirty
        if *self.dirty_i.read() {
            let insts = self.insts.read();
            let schema = self.schema_i.read();
            if insts.is_empty() {
                *self.packed_insts.write() = Vec::new();
                *self.dirty_i.write() = false;
            } else {
                let mut bytes = Vec::new();
                for ins in insts.iter() {
                    pack_instance(&mut bytes, ins, schema.as_ref().unwrap())?;
                }
                *self.packed_insts.write() = bytes;
                *self.dirty_i.write() = false;
            }
        }
        Ok(())
    }

    pub(crate) fn ensure_gpu(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(GpuOwned, DrawCounts), MeshError> {
        // Capture dirty flags before packing; ensure_packed() clears them
        let was_dirty_v = *self.dirty_v.read();
        let was_dirty_i = *self.dirty_i.read();

        self.ensure_packed()?;
        let pv = self.packed_verts.read();
        let pi = self.packed_insts.read();
        let idx = self.indices.read();
        let si = self.schema_i.read();

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
                let current_cap = g.instance_bytes;
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
                    Some((
                        buf,
                        (pi.len() as u32) / (si.as_ref().unwrap().stride as u32),
                    )),
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
                instance_bytes,
            });
        } else {
            // Update contents if needed without recreating buffers
            if let Some(g) = gpu.as_ref() {
                if was_dirty_v {
                    queue.write_buffer(&g.vertex_buffer, 0, &pv);
                    *self.dirty_v.write() = false;
                }
                if was_dirty_i {
                    if let Some((ref buf, _)) = g.instance_buffer {
                        // Safe to write since capacity check passed above
                        queue.write_buffer(buf, 0, &pi);
                    }
                    *self.dirty_i.write() = false;
                }
            }
        }

        let g = gpu.as_ref().unwrap();
        let refs = GpuOwned {
            vertex_buffer: g.vertex_buffer.clone(),
            index_buffer: g.index_buffer.clone(),
            instance_buffer: g.instance_buffer.as_ref().map(|(b, _)| b.clone()),
        };
        let override_count = *self.override_instances.read();
        let counts = DrawCounts {
            index_count: g.instance_buffer_len,
            instance_count: override_count
                .unwrap_or_else(|| g.instance_buffer.as_ref().map(|(_, c)| *c).unwrap_or(1)),
        };
        Ok((refs, counts))
    }
}

pub(crate) struct GpuOwned {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: Option<wgpu::Buffer>,
}

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

fn derive_vertex_schema(vertex: &Vertex) -> Result<Schema, MeshError> {
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
    Ok(Schema { stride, fields })
}

fn derive_instance_schema(i: &Instance) -> Result<Schema, MeshError> {
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
    Ok(Schema { stride, fields })
}

fn pack_vertex(out: &mut Vec<u8>, v: &Vertex, schema: &Schema) {
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

fn pack_instance(out: &mut Vec<u8>, i: &Instance, schema: &Schema) -> Result<(), MeshError> {
    for f in schema.fields.iter() {
        if let Some(val) = i.properties.get(&f.name) {
            out.extend_from_slice(&val.to_bytes());
        } else {
            out.extend(std::iter::repeat_n(0u8, f.size as usize));
        }
    }
    Ok(())
}
