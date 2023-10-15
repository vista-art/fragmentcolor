use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Globals {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GlobalsUniform {
    view_matrix: [[f32; 4]; 4],
}

impl Globals {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        viewport_width: u32,
        viewport_height: u32,
    ) -> Self {
        let temp_label = format!("Globals buffer");
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: &temp_label,
            contents: bytemuck::cast_slice(&[GlobalsUniform {
                view_matrix: [
                    [1.0 / (viewport_width as f32 / 2.0), 0.0, 0.0, 0.0],
                    [0.0, -1.0 / (viewport_height as f32 / 2.0), 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [-1.0, 1.0, 0.0, 1.0],
                ],
            }]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_label = format!("Globals bind group");
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: &bind_group_label,
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { bind_group, buffer }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenUniform {
    resolution: [f32; 2],
    antialiaser: f32,
    _padding: f32,
}

impl ScreenUniform {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            antialiaser: 2.0 / f32::min(width, height),
            resolution: [width, height],
            _padding: 0.0,
        }
    }
}
