use crate::{
    renderer::{Uniform, UniformOperations},
    shapes::Circle,
};
use enum_dispatch::enum_dispatch;
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

/// Represents objects that can be rendered in the screen
#[enum_dispatch]
pub trait RenderableTrait: Sized {
    /// A label must be provided by the implementor
    fn label(&self) -> String;
    /// A uniform definition must be provided by the implementor
    fn uniform(&self) -> Arc<RwLock<Uniform>>;
    /// The implementor must provide a way to update the uniform
    fn update(&mut self);
    /// Tells the renderer if the uniform should be updated
    fn should_update(&self) -> bool {
        true
    }
    /// The implementor must provide a way to convert the uniform
    /// into a raw bytes representation.
    fn uniform_bytes(&self) -> Vec<u8> {
        self.uniform().read().unwrap().bytes()
    }

    /// The renderer injects the GPU device instance to the Uniform
    /// and expects a raw bytes representation of its data as a buffer.
    fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer;
}

#[enum_dispatch(RenderableTrait)]
pub enum Renderable {
    Circle(Circle),
    //Rectangle(crate::shapes::Rectangle),
    //Text(crate::shapes::Text),
}

pub type RenderableRef = Arc<RwLock<Renderable>>;
pub type RenderableRefs = Vec<Arc<RwLock<Renderable>>>;

impl Debug for Renderable {
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
