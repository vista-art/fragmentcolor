use crate::renderer;
use std::{any::TypeId, marker::PhantomData, mem};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MeshId(pub u32);

/// Mesh is a GPU resource, not a Scene resource.
pub struct Mesh {
    pub buffer: wgpu::Buffer,
    // This is an slice because a Vertex might hold
    // multiple types of data (position, normal, etc)
    vertex_streams: Box<[VertexStream]>,
    pub index_stream: Option<IndexStream>,
    pub vertex_count: u32,
    pub bound_radius: f32,
}

pub struct IndexStream {
    pub offset: wgpu::BufferAddress,
    pub format: wgpu::IndexFormat,
    pub count: u32,
}

pub struct VertexStream {
    type_id: TypeId,
    pub offset: wgpu::BufferAddress,
    pub stride: wgpu::BufferAddress,
}

/// The original engine defines this struct as a
/// renamed hecs::Bundle implementor which holds
/// a reference to a Mesh.
///
/// This is the original description:
///   A freshly created Mesh that comes with metadata,
///   which is necessary to instantiate it.
///
///
#[derive(hecs::Bundle, hecs::DynamicBundleClone)]
pub struct Prototype {
    pub reference: MeshId,
    type_ids: Box<[TypeId]>,
    type_infos: Box<[hecs::TypeInfo]>,
}

// Apparently, this hack is there to enable this Prototype to be
// added to Scenes as references - so we can have the builder
// pattern of the original engine.
//
// When a user injects it in the scene as a reference, the Scene
// adds it to the hecs::World as a reference too, so it can return
// a builder which holds the same reference.
//
// The unusual bit of this pattern is that the "add" method of
// the scene, instead of returning an ID, will return a builder.
// Additionally, the "build" method of the builder, instead of
// returning a new instance of the type, implicitly injects the
// type into the Scene and returns this Prototype reference back.
unsafe impl<'a> hecs::DynamicBundle for &'a Prototype {
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

pub struct Vertex<T>(PhantomData<T>);

pub struct MeshBuilder<'a> {
    renderer: &'a mut renderer::Renderer,
    name: String,
    data: Vec<u8>, // could be moved up to the context
    index_stream: Option<IndexStream>,
    vertex_streams: Vec<VertexStream>,
    type_infos: Vec<hecs::TypeInfo>,
    vertex_count: usize,
    bound_radius: f32,
}

impl<'a> MeshBuilder<'a> {
    pub fn new(renderer: &'a mut renderer::Renderer) -> Self {
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

    pub fn name<'s>(&'a mut self, name: &str) -> &'s mut Self {
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

    pub fn build(&mut self) -> Prototype {
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

        let index = self
            .renderer
            .add_mesh(Mesh {
                buffer,
                index_stream: self.index_stream.take(),
                vertex_streams: mem::take(&mut self.vertex_streams).into_boxed_slice(),
                vertex_count: self.vertex_count as u32,
                bound_radius: self.bound_radius,
            })
            .expect("Could not build mesh");

        Prototype {
            reference: index,
            type_ids,
            type_infos: mem::take(&mut self.type_infos).into_boxed_slice(),
        }
    }
}
