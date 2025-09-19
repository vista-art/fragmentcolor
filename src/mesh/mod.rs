use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

pub mod error;
pub use error::*;

pub mod position;
pub use position::*;

pub mod vertex;
pub use vertex::*;

#[derive(Clone, Debug)]
#[lsp_doc("docs/api/core/mesh/mesh.md")]
pub struct Mesh {
    pub(crate) object: Arc<MeshObject>,
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
}

// -----------------------------
// Internals
// -----------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct VertexKey {
    pos: PosBits,
    props: Vec<(String, PropBits)>, // sorted by key
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
        let pos = match v.pos {
            Position::Pos2(p) => PosBits::P2([p[0].to_bits(), p[1].to_bits()]),
            Position::Pos3(p) => PosBits::P3([p[0].to_bits(), p[1].to_bits(), p[2].to_bits()]),
        };
        let mut props: Vec<(String, PropBits)> = v
            .props
            .iter()
            .map(|(k, val)| (k.clone(), PropBits::B(val.to_bytes())))
            .collect();
        props.sort_by(|a, b| a.0.cmp(&b.0));
        VertexKey { pos, props }
    }
}

#[derive(Debug)]
pub(crate) struct MeshObject {
    // CPU side storage
    verts: RwLock<Vec<Vertex>>, // original order
    insts: RwLock<Vec<Instance>>,
    // Derived, packed bytes
    packed_verts: RwLock<Vec<u8>>, // unique verts packed by schema
    packed_insts: RwLock<Vec<u8>>, // instances packed by schema
    indices: RwLock<Vec<u32>>,     // indices referencing unique verts

    // Schemas
    schema_v: RwLock<Option<Schema>>, // derived from first vertex
    schema_i: RwLock<Option<Schema>>, // derived from first instance

    // Dirty flags
    dirty_v: RwLock<bool>,
    dirty_i: RwLock<bool>,

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

    pub(crate) fn layout_signature(&self) -> u64 {
        let sv = self.schema_v.read().clone();
        let si = self.schema_i.read().clone();
        let mut hasher = Sha256::new();
        if let Some(s) = sv {
            for f in s.fields.iter() {
                hasher.update(f.name.as_bytes());
                hasher.update([format_code(f.fmt)]);
            }
        }
        hasher.update([0u8]);
        if let Some(s) = si {
            for f in s.fields.iter() {
                hasher.update(f.name.as_bytes());
                hasher.update([format_code(f.fmt)]);
            }
        }
        let h = hasher.finalize();
        u64::from_le_bytes([h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]])
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
    ) -> Result<(GpuOwned, DrawCounts, Layouts), MeshError> {
        self.ensure_packed()?;
        let pv = self.packed_verts.read();
        let pi = self.packed_insts.read();
        let idx = self.indices.read();
        let sv = self.schema_v.read();
        let si = self.schema_i.read();

        // Create or grow buffers
        let mut gpu = self.gpu.write();
        let need_new = match gpu.as_ref() {
            None => true,
            Some(g) => g.instance_buffer_len as usize != idx.len(),
        } || *self.dirty_v.read()
            || *self.dirty_i.read();

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

            let instance_buffer = if !pi.is_empty() {
                let buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Mesh Instance Buffer"),
                    size: pi.len() as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                queue.write_buffer(&buf, 0, &pi);
                Some((
                    buf,
                    (pi.len() as u32) / (si.as_ref().unwrap().stride as u32),
                ))
            } else {
                None
            };

            gpu.replace(GpuStreams {
                vertex_buffer,
                index_buffer,
                instance_buffer_len: idx.len() as u32,
                instance_buffer,
            });
        } else {
            // Update contents if needed
            if let Some(g) = gpu.as_ref() {
                if *self.dirty_v.read() {
                    queue.write_buffer(&g.vertex_buffer, 0, &pv);
                    *self.dirty_v.write() = false;
                }
                if *self.dirty_i.read() {
                    if let Some((ref buf, _)) = g.instance_buffer {
                        queue.write_buffer(buf, 0, &pi);
                    }
                    *self.dirty_i.write() = false;
                }
            }
        }

        let g = gpu.as_ref().unwrap();
        let layouts = build_layouts(sv.as_ref().unwrap(), si.as_ref());
        let refs = GpuOwned {
            vertex_buffer: g.vertex_buffer.clone(),
            index_buffer: g.index_buffer.clone(),
            instance_buffer: g.instance_buffer.as_ref().map(|(b, _)| b.clone()),
        };
        let counts = DrawCounts {
            index_count: g.instance_buffer_len,
            instance_count: g.instance_buffer.as_ref().map(|(_, c)| *c).unwrap_or(1),
        };
        Ok((refs, counts, layouts))
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

pub(crate) struct Layouts {
    pub vertex: wgpu::VertexBufferLayout<'static>,
    pub instance: Option<wgpu::VertexBufferLayout<'static>>,
}

fn derive_vertex_schema(v: &Vertex) -> Result<Schema, MeshError> {
    let mut fields: Vec<Field> = Vec::new();
    // position first at location 0
    match v.pos {
        Position::Pos2(_) => fields.push(Field {
            name: "position2".into(),
            fmt: wgpu::VertexFormat::Float32x2,
            size: 8,
        }),
        Position::Pos3(_) => fields.push(Field {
            name: "position3".into(),
            fmt: wgpu::VertexFormat::Float32x3,
            size: 12,
        }),
    }
    // uv, color, then others sorted
    if let Some(VertexValue::F32x2(_)) = v.props.get("uv") {
        fields.push(Field {
            name: "uv".into(),
            fmt: wgpu::VertexFormat::Float32x2,
            size: 8,
        });
    }
    if let Some(VertexValue::F32x4(_)) = v.props.get("color") {
        fields.push(Field {
            name: "color".into(),
            fmt: wgpu::VertexFormat::Float32x4,
            size: 16,
        });
    }
    let mut rest: Vec<(&String, &VertexValue)> = v
        .props
        .iter()
        .filter(|(k, _)| k.as_str() != "uv" && k.as_str() != "color")
        .collect();
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
    // If position copied over, keep order uv,color,rest
    if let Some(VertexValue::F32x2(_)) = i.props.get("position2") {
        fields.push(Field {
            name: "position2".into(),
            fmt: wgpu::VertexFormat::Float32x2,
            size: 8,
        });
    }
    if let Some(VertexValue::F32x3(_)) = i.props.get("position3") {
        fields.push(Field {
            name: "position3".into(),
            fmt: wgpu::VertexFormat::Float32x3,
            size: 12,
        });
    }
    if let Some(VertexValue::F32x2(_)) = i.props.get("uv") {
        fields.push(Field {
            name: "uv".into(),
            fmt: wgpu::VertexFormat::Float32x2,
            size: 8,
        });
    }
    if let Some(VertexValue::F32x4(_)) = i.props.get("color") {
        fields.push(Field {
            name: "color".into(),
            fmt: wgpu::VertexFormat::Float32x4,
            size: 16,
        });
    }
    let mut rest: Vec<(&String, &VertexValue)> = i
        .props
        .iter()
        .filter(|(k, _)| !matches!(k.as_str(), "position2" | "position3" | "uv" | "color"))
        .collect();
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
            "position2" => {
                if let Position::Pos2(p) = v.pos {
                    out.extend_from_slice(bytemuck::cast_slice(&p));
                } else {
                    out.extend_from_slice(&[0; 8]);
                }
            }
            "position3" => {
                if let Position::Pos3(p) = v.pos {
                    out.extend_from_slice(bytemuck::cast_slice(&p));
                } else {
                    out.extend_from_slice(&[0; 12]);
                }
            }
            name => {
                if let Some(val) = v.props.get(name) {
                    out.extend_from_slice(&val.to_bytes());
                } else {
                    // zero-fill absent optional props
                    out.extend(std::iter::repeat_n(0u8, f.size as usize));
                }
            }
        }
    }
}

fn pack_instance(out: &mut Vec<u8>, i: &Instance, schema: &Schema) -> Result<(), MeshError> {
    for f in schema.fields.iter() {
        if let Some(val) = i.props.get(&f.name) {
            out.extend_from_slice(&val.to_bytes());
        } else {
            out.extend(std::iter::repeat_n(0u8, f.size as usize));
        }
    }
    Ok(())
}

pub(crate) fn build_layouts(v: &Schema, i: Option<&Schema>) -> Layouts {
    let (attrs_v, stride_v) = to_attrs(v, 0);
    let vertex = wgpu::VertexBufferLayout {
        array_stride: stride_v,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: Box::leak(attrs_v.into_boxed_slice()),
    };
    let instance = i.map(|s| {
        let (attrs_i, stride_i) = to_attrs(s, v.fields.len() as u32);
        wgpu::VertexBufferLayout {
            array_stride: stride_i,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Box::leak(attrs_i.into_boxed_slice()),
        }
    });
    Layouts { vertex, instance }
}

fn to_attrs(schema: &Schema, base_loc: u32) -> (Vec<wgpu::VertexAttribute>, u64) {
    let mut ofs = 0u64;
    let mut attrs = Vec::new();
    for (i, f) in schema.fields.iter().enumerate() {
        attrs.push(wgpu::VertexAttribute {
            format: f.fmt,
            offset: ofs,
            shader_location: base_loc + i as u32,
        });
        ofs += f.size;
    }
    (attrs, ofs)
}

fn format_code(fmt: wgpu::VertexFormat) -> u8 {
    use wgpu::VertexFormat as F;
    match fmt {
        F::Float32 => 1,
        F::Float32x2 => 2,
        F::Float32x3 => 3,
        F::Float32x4 => 4,
        F::Uint32 => 5,
        F::Uint32x2 => 6,
        F::Uint32x3 => 7,
        F::Uint32x4 => 8,
        F::Sint32 => 9,
        F::Sint32x2 => 10,
        F::Sint32x3 => 11,
        F::Sint32x4 => 12,
        _ => 0,
    }
}
