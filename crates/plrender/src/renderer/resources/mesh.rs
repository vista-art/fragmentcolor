use crate::geometry::vertex::Vertex;
use crate::renderer;
use std::any::TypeId;
use std::mem;
use wgpu::util::DeviceExt;

/// A unique identifier for a Mesh that
/// is already loaded into the Renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MeshId(pub(super) u32);

/// Metadata about a loaded Mesh resource.
///
/// This object is the link between the Scene
/// and the Renderer. It is used as the input
/// to create a Renderable component.
///
/// A MeshPrototype is created after inserting a Mesh
/// resource into the Renderer, which returns a MeshId.
/// The Prototype holds the MeshId and a list of TypeIds
/// and TypeInfos about the actual Mesh vertex layout.
///
/// The existence of a MeshPrototype means that
/// a Mesh is already loaded into the Renderer,
/// and the Prototype is a reference to it.
#[derive(hecs::Bundle, hecs::DynamicBundleClone)]
pub struct MeshPrototype {
    pub id: MeshId,
    pub(crate) type_ids: Box<[std::any::TypeId]>,
    pub(crate) type_infos: Box<[hecs::TypeInfo]>,
}

/// Makes it possible to use a Reference to a MeshPrototype
/// as a hecs::Bundle. Without this, we can only use concrete
/// types and it breaks the implementation of our asset loaders.
unsafe impl<'a> hecs::DynamicBundle for &'a MeshPrototype {
    fn with_ids<T>(&self, f: impl FnOnce(&[TypeId]) -> T) -> T {
        f(&self.type_ids)
    }
    fn type_info(&self) -> Vec<hecs::TypeInfo> {
        self.type_infos.to_vec()
    }
    unsafe fn put(self, mut f: impl FnMut(*mut u8, hecs::TypeInfo)) {
        const DUMMY_SIZE: usize = 1;
        let mut v = [0u8; DUMMY_SIZE];
        assert!(mem::size_of::<Vertex<()>>() <= DUMMY_SIZE);
        for ts in self.type_infos.iter() {
            f(v.as_mut_ptr(), ts.clone());
        }
    }
}

/// Mesh is a GPU resource, not a Scene resource.
#[derive(Debug)]
pub struct Mesh {
    // A Vertex might hold multiple types of
    // data (position, normal, color, etc)
    vertices: Box<[VertexData]>,
    pub buffer: wgpu::Buffer,
    pub vertex_ids: Option<VertexIds>,
    pub vertex_count: u32,
    pub bound_radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct VertexIds {
    pub offset: wgpu::BufferAddress,
    pub format: wgpu::IndexFormat,
    pub count: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct VertexData {
    type_id: std::any::TypeId,
    pub offset: wgpu::BufferAddress,
    pub stride: wgpu::BufferAddress,
}

impl Mesh {
    pub fn vertex_data<VertexType: 'static>(&self) -> Option<&VertexData> {
        self.vertices
            .iter()
            .find(|vertex| vertex.type_id == std::any::TypeId::of::<VertexType>())
    }

    pub fn vertex_slice<T: 'static>(&self) -> wgpu::BufferSlice {
        let data = self.vertex_data::<T>().unwrap();
        self.buffer.slice(data.offset..)
    }
}

pub struct MeshBuilder<'r> {
    renderer: &'r mut renderer::Renderer,
    name: String,
    data: Vec<u8>,
    vertex_ids: Option<VertexIds>,
    vertices: Vec<VertexData>,
    type_infos: Vec<hecs::TypeInfo>,
    vertex_count: usize,
    bound_radius: f32,
}

impl<'r> MeshBuilder<'r> {
    pub fn new(renderer: &'r mut renderer::Renderer) -> Self {
        Self {
            renderer,
            name: String::new(),
            data: Vec::new(),
            vertex_ids: None,
            vertices: Vec::new(),
            type_infos: Vec::new(),
            vertex_count: 0,
            bound_radius: 0.0,
        }
    }

    pub fn name<'s>(&'r mut self, name: &str) -> &'s mut Self {
        self.name = name.to_string();
        self
    }

    fn append<T: bytemuck::Pod>(&mut self, data: &[T]) -> wgpu::BufferAddress {
        let offset = self.data.len();
        self.data.extend(bytemuck::cast_slice(data));
        offset as _
    }

    pub fn index<'s>(&'s mut self, data: &[u16]) -> &'s mut Self {
        assert!(self.vertex_ids.is_none());
        let offset = self.append(data);
        self.vertex_ids = Some(VertexIds {
            offset,
            format: wgpu::IndexFormat::Uint16,
            count: data.len() as u32,
        });
        self
    }

    pub fn vertex<'s, T: bytemuck::Pod>(&'s mut self, data: &[T]) -> &'s mut Self {
        let offset = self.append(data);
        if self.vertex_count == 0 {
            self.vertex_count = data.len();
        } else {
            assert_eq!(self.vertex_count, data.len());
        }
        self.vertices.push(VertexData {
            type_id: std::any::TypeId::of::<T>(),
            offset,
            stride: mem::size_of::<T>() as _,
        });
        self.type_infos.push(hecs::TypeInfo::of::<Vertex<T>>());
        self
    }

    pub fn radius(&mut self, radius: f32) -> &mut Self {
        self.bound_radius = radius;
        self
    }

    pub fn build(&mut self) -> MeshPrototype {
        let mut usage = wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX;
        usage.set(wgpu::BufferUsages::INDEX, self.vertex_ids.is_some());
        let buffer = self
            .renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: if self.name.is_empty() {
                    None
                } else {
                    Some(&self.name)
                },
                contents: &self.data,
                usage,
            });

        let type_ids = self
            .vertices
            .iter()
            .map(|vs| vs.type_id)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let mesh_id = self.renderer.add_mesh(Mesh {
            buffer,
            vertex_ids: self.vertex_ids.take(),
            vertices: mem::take(&mut self.vertices).into_boxed_slice(),
            vertex_count: self.vertex_count as u32,
            bound_radius: self.bound_radius,
        });

        MeshPrototype {
            id: mesh_id,
            type_ids,
            type_infos: mem::take(&mut self.type_infos).into_boxed_slice(),
        }
    }
}
