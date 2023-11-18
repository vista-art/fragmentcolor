use crate::scene::vertex::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl Vertex for ShapeVertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ShapeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct Shape {
    pub vertices: &'static [ShapeVertex],
    pub indices: &'static [u16],
}

pub const QUAD: Shape = Shape {
    vertices: QUAD_VERTICES,
    indices: QUAD_INDICES,
};

const QUAD_VERTICES: &[ShapeVertex] = &[
    ShapeVertex {
        position: [-1.0, -1.0, 0.0],
        color: [1.0, 0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    ShapeVertex {
        position: [-1.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0, 1.0],
        tex_coords: [0.0, 1.0],
    },
    ShapeVertex {
        position: [1.0, -1.0, 0.0],
        color: [0.0, 0.0, 1.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    ShapeVertex {
        position: [1.0, 1.0, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
];

const QUAD_INDICES: &[u16] = &[0, 2, 1, 2, 3, 1];
