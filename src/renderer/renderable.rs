//use crate::renderer::Uniform;
use std::{any::Any, cell::RefCell, fmt::Debug, sync::Arc};

/// The raw Uniform bytes that will be sent to the GPU
pub trait Uniform<T>: Debug + Default + Copy + Clone + bytemuck::Pod + bytemuck::Zeroable {
    fn new() -> Self {
        Self::default()
    }
    fn update(&mut self, data: &T);
    fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer;
    // fn bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
    // fn bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout;
}
/*
let renderable_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some(format!("{} Uniform Buffer", &label).as_str()),
    contents: bytemuck::cast_slice(&[*renderable_uniform]),
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
});

let renderable_bind_group_layout =
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: i,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some(format!("{} Bind Group Layout", &label)),
    });

let renderable_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    layout: &renderable_bind_group_layout,
    entries: &[wgpu::BindGroupEntry {
        binding: i,
        resource: renderable_buffer.as_entire_binding(),
    }],
    label: Some(format!("{} Bind Group", &label)),
});
*/

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
        impl crate::renderer::Uniform<$owner> for $name {
            fn update(&mut self, _data: &$owner) {
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
            // fn bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
            //     let layout = self.bind_group_layout(device);
            //     let buffer = self.buffer(device);
            //     device.create_bind_group(&wgpu::BindGroupDescriptor {
            //         layout,
            //         entries: &[wgpu::BindGroupEntry {
            //             binding: 0,
            //             resource: buffer.as_entire_binding(),
            //         }],
            //         label: Some(format!("{} Bind Group", stringify!($name))),
            //     })
            // }
            // fn bind_group_layout(&self, device: &wgpu::Device) -> wgou::BindGroupLayout {
            //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            //         entries: &[wgpu::BindGroupLayoutEntry {
            //             binding: 0,
            //             visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            //             ty: wgpu::BindingType::Buffer {
            //                 ty: wgpu::BufferBindingType::Uniform,
            //                 has_dynamic_offset: false,
            //                 min_binding_size: None,
            //             },
            //             count: None,
            //         }],
            //         label: Some(format!("{} Bind Group Layout", stringify!($name))),
            //     })
            // }
        }
    };
}

/// Represents objects that can be rendered in the screen
pub trait Renderable: Sized {
    type U: Uniform<Self> + 'static;

    /// A label must be provided by the implementor
    fn label(&self) -> String;
    /// A uniform definition must be provided by the implementor
    fn uniform(&self) -> Arc<RefCell<Self::U>>;
    /// The implementor must provide a way to update the uniform
    fn update(&self);

    // The renderer expects getters and setters for
    // buffers, bind groups, and bind group layouts.
    //
    // The implementor does not need to build them,
    // the renderer will do that from the uniform
    // definition and then call the setters.
    fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer;
    //fn set_buffer(&self, buffer: wgpu::Buffer);

    //fn bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
    //fn set_bind_group(&self, bind_group: wgpu::BindGroup);

    //fn bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout;
    //fn set_bind_group_layout(&self, bind_group_layout: wgpu::BindGroupLayout);
}

pub trait RenderableOperations {
    fn as_any(&self) -> &dyn Any;
    fn label(&self) -> String;
    fn uniform(&self) -> Arc<RefCell<dyn Any>>;
    fn update(&self);
    fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer;
    //fn set_buffer(&self, buffer: wgpu::Buffer);
    //fn bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
    //fn set_bind_group(&self, bind_group: wgpu::BindGroup);
    //fn bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout;
    //fn set_bind_group_layout(&self, bind_group_layout: wgpu::BindGroupLayout);
}

impl<T, U> RenderableOperations for T
where
    T: Renderable<U = U> + 'static,
    U: Uniform<Self>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn label(&self) -> String {
        self.label()
    }

    fn uniform(&self) -> Arc<RefCell<dyn Any>> {
        self.uniform()
    }

    fn update(&self) {
        self.update()
    }

    fn buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        self.buffer(device)
    }

    // fn set_buffer(&self, buffer: wgpu::Buffer) {
    //     self.set_buffer(buffer)
    // }

    // fn bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
    //     self.bind_group(device)
    // }

    // fn set_bind_group(&self, bind_group: wgpu::BindGroup) {
    //     self.set_bind_group(bind_group)
    // }

    // fn bind_group_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
    //     self.bind_group_layout(device)
    // }

    // fn set_bind_group_layout(&self, bind_group_layout: wgpu::BindGroupLayout) {
    //     self.set_bind_group_layout(bind_group_layout)
    // }
}

//pub type RenderableFor<T> = dyn Renderable<U = T>;
pub type AnyRenderable = Box<dyn RenderableOperations>;
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
