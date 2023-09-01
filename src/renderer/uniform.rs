use crate::renderer::AnyRenderable;
use enum_dispatch::enum_dispatch;
use std::fmt::Debug;

/// The raw Uniform bytes that will be sent to the GPU
pub trait Uniform: Debug + Default + Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {}

#[enum_dispatch]
pub trait UniformOperations: Default + Debug {
    fn update(&mut self, data: &AnyRenderable);
    fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer;
    fn bytes(&self) -> Vec<u8>;
}

#[derive(Debug)]
#[enum_dispatch(UniformOperations)]
pub enum AnyUniform {
    Circle(crate::shapes::CircleUniform),
    //Rectangle(crate::shapes::RectangleUniform),
    //Text(crate::shapes::TextUniform),
}

impl Default for AnyUniform {
    fn default() -> Self {
        AnyUniform::Circle(crate::shapes::CircleUniform::default())
    }
}

#[macro_export]
macro_rules! uniform {
    ($name:ident for $owner:ty {
        $($field:ident: $type:ty),* $(,)?
    }) => {
        #[repr(C)]
        #[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $name {
            $($field: $type),*
        }
        impl crate::renderer::Uniform for $name {}
        impl crate::renderer::UniformOperations for $name {
            fn update(&mut self, _data: &crate::renderer::AnyRenderable) {
                unimplemented!("User should implement uniform update() for {}", stringify!($name));
            }
            fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
                use wgpu::util::DeviceExt;
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(format!("{} Uniform Buffer", stringify!($name)).as_str()),
                    contents: bytemuck::cast_slice(&[*self]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
            }
            fn bytes(&self, ) -> std::vec::Vec<u8> {
                let bytes = std::vec::Vec::from(
                    bytemuck::cast_slice(&[*self])
                );
                bytes
            }
        }
    };
}
