use cfg_if::cfg_if;

pub trait Vertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static>;
}

pub struct Shape {
    pub vertices: &'static [ShapeVertex],
    pub indices: &'static [u16],
}

cfg_if! { if #[cfg(feature = "texture")] {
    #[repr(C)]
    #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct ShapeVertex {
        pub position: [f32; 3],
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
                        format: wgpu::VertexFormat::Float32x2,
                    },
                ],
            }
        }
    }

    pub const TEXTURED_PENTAGON: Shape = Shape {
        vertices: PENTAGON_VERTICES,
        indices: PENTAGON_INDICES,
    };

    const PENTAGON_VERTICES: &[ShapeVertex] = &[
        ShapeVertex {
            position: [-0.0868241, 0.49240386, 0.0],
            tex_coords: [0.4131759, 1.0 - 0.99240386],
        }, // A
        ShapeVertex {
            position: [-0.49513406, 0.06958647, 0.0],
            tex_coords: [0.0048659444, 1.0 - 0.56958647],
        }, // B
        ShapeVertex {
            position: [-0.21918549, -0.44939706, 0.0],
            tex_coords: [0.28081453, 1.0 - 0.05060294],
        }, // C
        ShapeVertex {
            position: [0.35966998, -0.3473291, 0.0],
            tex_coords: [0.85967, 1.0 - 0.1526709],
        }, // D
        ShapeVertex {
            position: [0.44147372, 0.2347359, 0.0],
            tex_coords: [0.9414737, 1.0 - 0.7347359],
        }, // E
    ];

    const PENTAGON_INDICES: &[u16] = &[
        0, 1, 4, // ABE
        1, 2, 4, // BCE
        2, 3, 4, // CDE
    ];


} else {
    #[repr(C)]
    #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
    pub struct ShapeVertex {
        pub position: [f32; 3],
    }

    impl Vertex for ShapeVertex {
        fn descriptor() -> wgpu::VertexBufferLayout<'static> {
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ShapeVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                }],
            }
        }
    }

    pub const FULL_SCREEN_QUAD: Shape = Shape {
        vertices: FULL_SCREEN_QUAD_VERTICES,
        indices: FULL_SCREEN_QUAD_INDICES,
    };

    const FULL_SCREEN_QUAD_VERTICES: &[ShapeVertex] = &[
        ShapeVertex {
            position: [-1.0, -1.0, 0.0],
        },
        ShapeVertex {
            position: [-1.0, 1.0, 0.0],
        },
        ShapeVertex {
            position: [1.0, -1.0, 0.0],
        },
        ShapeVertex {
            position: [1.0, 1.0, 0.0],
        },
    ];

    const FULL_SCREEN_QUAD_INDICES: &[u16] = &[0, 2, 1, 2, 3, 1];
}}
