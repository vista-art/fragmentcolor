// use std::fmt::Debug;

// /// The raw Uniform bytes that will be sent to the GPU
// pub trait Uniform: Debug + Default + Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {}

// #[macro_export]
// macro_rules! uniform {
//     ($name:ident {
//         $($field:ident: $type:ty),* $(,)?
//     }) => {
//         #[repr(C)]
//         #[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
//         pub struct $name {
//             $($field: $type),*
//         }
//         impl crate::renderer::uniform::Uniform for $name {}
//     };
// }
