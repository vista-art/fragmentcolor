use crate::{renderer::AnyUniform, shapes::Circle};
use enum_dispatch::enum_dispatch;
use std::{cell::RefCell, fmt::Debug, sync::Arc};

use super::UniformOperations;

/// Represents objects that can be rendered in the screen
#[enum_dispatch]
pub trait Renderable: Sized {
    /// A label must be provided by the implementor
    fn label(&self) -> String;
    /// A uniform definition must be provided by the implementor
    fn uniform(&self) -> Arc<RefCell<AnyUniform>>;
    /// The implementor must provide a way to update the uniform
    fn update(&self);
    /// The implementor must provide a way to convert the uniform
    /// into a raw bytes representation.
    fn uniform_bytes(&self) -> Vec<u8> {
        self.uniform().borrow().bytes()
    }

    /// The renderer injects the GPU device instance to the Uniform
    /// and expects a raw bytes representation of its data as a buffer.
    fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer;
}

#[enum_dispatch(Renderable)]
pub enum AnyRenderable {
    Circle(Circle),
    //Rectangle(crate::shapes::Rectangle),
    //Text(crate::shapes::Text),
}

pub type Renderables = Vec<AnyRenderable>;

impl Debug for AnyRenderable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Renderable{{buffer: {}, uniform: {}, bind_group: {}}}",
            "buffer formatter not implemented",
            "uniform formatter not implemented",
            "bind_group formatter not implemented"
        )
    }
}
