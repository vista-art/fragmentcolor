use crate::geometry::vertex::Vertex;
use crate::renderer;
use std::{any::TypeId, mem};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MeshId(pub(super) u32);

// @TODO this should be removed.
//       there should be a more direct usage of hecs and ECS,
//       and a more clear relationship between scene entities
//       and renderer Resources. Perhaps this thing that holds
//       the MeshId is what you cal "Renderables" component.
//       Any entity with a Renderable component will have a
//       mesh registered in the Renderer, and will hold its MeshId.

/// A freshly created Mesh that comes with metadata,
/// which is necessary to instantiate it.
#[derive(hecs::Bundle, hecs::DynamicBundleClone)]
pub struct MeshPrototype {
    pub id: MeshId,
    pub(crate) type_ids: Box<[TypeId]>,
    pub(crate) type_infos: Box<[hecs::TypeInfo]>,
}
/// Mesh is a GPU resource, not a Scene resource.
#[derive(Debug)]
pub struct Mesh {
    pub buffer: wgpu::Buffer,
    // This is a slice because a Vertex might hold
    // multiple types of data (position, normal, etc)
    vertex_streams: Box<[VertexStream]>,
    pub index_stream: Option<IndexStream>,
    pub vertex_count: u32,
    pub bound_radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct IndexStream {
    pub offset: wgpu::BufferAddress,
    pub format: wgpu::IndexFormat,
    pub count: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct VertexStream {
    type_id: TypeId,
    pub offset: wgpu::BufferAddress,
    pub stride: wgpu::BufferAddress,
}

impl Mesh {
    pub fn vertex_stream<T: 'static>(&self) -> Option<&VertexStream> {
        self.vertex_streams
            .iter()
            .find(|vs| vs.type_id == TypeId::of::<T>())
    }

    pub fn vertex_slice<T: 'static>(&self) -> wgpu::BufferSlice {
        let stream = self.vertex_stream::<T>().unwrap();
        self.buffer.slice(stream.offset..)
    }
}

pub struct MeshBuilder<'r> {
    renderer: &'r mut renderer::Renderer,
    name: String,
    data: Vec<u8>, // could be moved up to the context
    index_stream: Option<IndexStream>,
    vertex_streams: Vec<VertexStream>,
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
            index_stream: None,
            vertex_streams: Vec::new(),
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
        assert!(self.index_stream.is_none());
        let offset = self.append(data);
        self.index_stream = Some(IndexStream {
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
        self.vertex_streams.push(VertexStream {
            type_id: TypeId::of::<T>(),
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
        usage.set(wgpu::BufferUsages::INDEX, self.index_stream.is_some());
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
            .vertex_streams
            .iter()
            .map(|vs| vs.type_id)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let index = self.renderer.add_mesh(Mesh {
            buffer,
            index_stream: self.index_stream.take(),
            vertex_streams: mem::take(&mut self.vertex_streams).into_boxed_slice(),
            vertex_count: self.vertex_count as u32,
            bound_radius: self.bound_radius,
        });

        MeshPrototype {
            id: index,
            type_ids,
            type_infos: mem::take(&mut self.type_infos).into_boxed_slice(),
        }
    }
}
