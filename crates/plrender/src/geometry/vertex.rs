use bytemuck::{Pod, Zeroable};
use std::{marker::PhantomData, mem};

pub struct Vertex<T>(PhantomData<T>);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Position(pub [f32; 3]);

impl Position {
    pub(crate) const fn layout<const LOCATION: u32>() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: LOCATION,
            }],
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Normal(pub [f32; 3]);

impl Normal {
    pub(crate) const fn layout<const LOCATION: u32>() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: LOCATION,
            }],
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TexCoords(pub [u16; 2]);

impl TexCoords {
    pub(crate) const fn layout<const LOCATION: u32>() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Unorm16x2,
                offset: 0,
                shader_location: LOCATION,
            }],
        }
    }
}
