use crate::{
    InitializationError, PassInput, PassObject, ShaderError, ShaderHash, Target, TargetFrame,
    shader::Uniform,
};
use parking_lot::RwLock;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
pub type Commands = Vec<wgpu::CommandBuffer>;

mod features;

pub mod platform;
pub use platform::*;

mod buffer_pool;
use buffer_pool::BufferPool;

#[cfg(feature = "python")]
pub use features::python::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub trait Renderable {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject>;
}

#[derive(Debug)]
struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

#[derive(Debug)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Renderer {
    instance: wgpu::Instance,
    adapter: RwLock<Option<wgpu::Adapter>>,

    /// The graphics context is lazily initialized when
    /// create_target() or render_image() is called.
    context: RwLock<Option<Arc<RenderContext>>>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            instance: wgpu::Instance::new(&wgpu::InstanceDescriptor::default()),
            adapter: RwLock::new(None),
            context: RwLock::new(None),
        }
    }

    pub fn from_instance(instance: wgpu::Instance) -> Self {
        Renderer {
            instance,
            adapter: RwLock::new(None),
            context: RwLock::new(None),
        }
    }

    pub(crate) async fn create_surface<'window>(
        &self,
        handle: impl Into<wgpu::SurfaceTarget<'window>>,
        size: wgpu::Extent3d,
    ) -> Result<
        (
            Arc<RenderContext>,
            wgpu::Surface<'window>,
            wgpu::SurfaceConfiguration,
        ),
        InitializationError,
    > {
        let surface = self.instance.create_surface(handle)?;

        let context = if let Some(context) = self.context.read().as_ref() {
            context.clone()
        } else {
            let adapter =
                crate::platform::all::request_adapter(&self.instance, Some(&surface)).await?;
            let (device, queue) = crate::platform::all::request_device(&adapter).await?;
            let context = Arc::new(RenderContext::new(device, queue));

            self.adapter.write().replace(adapter);
            self.context.write().replace(context.clone());

            context
        };

        let adapter = self.adapter.read();
        let config = crate::platform::all::configure_surface(
            &context.device,
            adapter.as_ref().unwrap(),
            &surface,
            &size,
        );

        // @TODO consider returning a RenderTarget or WindowTarget
        Ok((context, surface, config))
    }

    /// Creates a headless Context
    // pub async fn render_image(&self) -> Result<Vec<u8>, InitializationError> {
    //     let _context = if let Some(context) = self.context.read().as_ref() {
    //         context.clone()
    //     } else {
    //         let adapter = crate::platform::all::request_headless_adapter(&self.instance).await?;
    //         let (device, queue) = crate::platform::all::request_device(&adapter).await?;
    //         let context = Arc::new(RenderContext::init(device, queue));

    //         self.adapter.write().replace(adapter);
    //         self.context.write().replace(context.clone());

    //         context
    //     };

    //     // @TODO
    // }

    pub fn render(
        &self,
        renderable: &impl Renderable,
        target: &impl Target,
    ) -> Result<(), ShaderError> {
        if let Some(context) = self.context.read().as_ref() {
            context.render(renderable, target)
        } else {
            Err(ShaderError::ContextNotInitialized())
        }
    }
}

#[derive(Debug)]
/// Draws things on the screen or a texture.
///
/// It owns and manages all GPU resources, serving as the
/// main graphics context provider for the application.
pub struct RenderContext {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    render_pipelines: RwLock<HashMap<(ShaderHash, wgpu::TextureFormat), RenderPipeline>>,
    buffer_pool: RwLock<BufferPool>,
    //
    // @TODO
    // _compute_pipelines: RwLock<HashMap<String, wgpu::ComputePipeline>>,
    // _textures: RwLock<HashMap<String, wgpu::Texture>>,
    // _samplers: RwLock<HashMap<String, wgpu::Sampler>>,
}

impl RenderContext {
    /// Creates a new Context with the given device and queue.
    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let buffer_pool = BufferPool::new_uniform_pool("Uniform Buffer Pool", &device);

        RenderContext {
            device,
            queue,

            render_pipelines: RwLock::new(HashMap::new()),
            buffer_pool: RwLock::new(buffer_pool),
        }
    }

    /// Renders a Frame or Shader to a Target.
    fn render(
        &self,
        renderable: &impl Renderable,
        target: &impl Target,
    ) -> Result<(), ShaderError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        let frame = target.get_current_frame()?;

        for pass in renderable.passes() {
            if pass.is_compute() {
                self.process_compute_pass(&mut encoder, pass)?
            } else {
                self.process_render_pass(&mut encoder, pass, frame.as_ref())?
            }
        }

        self.queue.submit(Some(encoder.finish()));

        if frame.auto_present() {
            frame.present();
        }

        Ok(())
    }
}

impl RenderContext {
    fn process_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pass: &PassObject,
        frame: &dyn TargetFrame,
    ) -> Result<(), ShaderError> {
        self.buffer_pool.write().reset();

        let load_op = match pass.get_input() {
            PassInput::Clear(color) => wgpu::LoadOp::Clear(color.into()),
            PassInput::Load() => wgpu::LoadOp::Load,
        };

        let attachments = &[Some(wgpu::RenderPassColorAttachment {
            view: frame.view(),
            resolve_target: None,
            ops: wgpu::Operations {
                load: load_op,
                store: wgpu::StoreOp::Store,
            },
        })];

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("Render Pass: {}", pass.name.clone())),
            color_attachments: attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // render_pass.set_blend_constant(wgpu::Color::WHITE);

        let required_size = *pass.required_buffer_size.read();
        self.buffer_pool
            .write()
            .ensure_capacity(required_size, &self.device);

        for shader in pass.shaders.read().iter() {
            let format = frame.format();
            let mut pipelines = self.render_pipelines.write();
            let cached = pipelines.entry((shader.hash, format)).or_insert_with(|| {
                let layouts =
                    create_bind_group_layouts(&self.device, &shader.storage.read().uniforms);
                let pipeline = create_render_pipeline(&self.device, &layouts, shader, format);

                RenderPipeline {
                    pipeline,
                    bind_group_layouts: layouts.values().cloned().collect(),
                }
            });

            let mut bind_group_entries: HashMap<u32, Vec<wgpu::BindGroupEntry>> = HashMap::new();
            let mut buffer_locations = Vec::new();

            for name in &shader.list_uniforms() {
                let uniform = shader.get_uniform(name)?;

                let storage = shader.storage.read();
                let bytes = storage
                    .get_bytes(name)
                    .ok_or(ShaderError::UniformNotFound(name.clone()))?;

                let buffer_location = {
                    let mut buffer_pool = self.buffer_pool.write();
                    buffer_pool.upload(bytes, &self.queue)
                };

                buffer_locations.push((uniform, buffer_location));
            }

            let buffer_pool = self.buffer_pool.read();
            for (uniform, location) in buffer_locations {
                let binding = buffer_pool.get_binding(location);

                bind_group_entries
                    .entry(uniform.group)
                    .or_default()
                    .push(wgpu::BindGroupEntry {
                        binding: uniform.binding,
                        resource: wgpu::BindingResource::Buffer(binding),
                    });
            }

            let mut bind_groups = Vec::new();
            for (group, entries) in bind_group_entries {
                let layout = &cached.bind_group_layouts[group as usize];
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &entries,
                    label: Some(&format!("Bind Group for group: {}", group)),
                });
                bind_groups.push(bind_group);
            }

            render_pass.set_pipeline(&cached.pipeline);
            for (i, bind_group) in bind_groups.iter().enumerate() {
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            }
            // render_pass.set_blend_constant(color);
            render_pass.draw(0..3, 0..1); // Fullscreen triangle
        }
        Ok(())
    }

    fn process_compute_pass(
        &self,
        _encoder: &mut wgpu::CommandEncoder,
        _pass: &PassObject,
    ) -> Result<(), ShaderError> {
        Ok(()) // @TODO later
    }
}

fn create_bind_group_layouts(
    device: &wgpu::Device,
    uniforms: &HashMap<String, (u32, u32, Uniform)>,
) -> HashMap<u32, wgpu::BindGroupLayout> {
    let mut group_entries = HashMap::new();
    for (_, _, uniform) in uniforms.values() {
        if uniform.name.contains('.') {
            continue;
        }

        let entry = wgpu::BindGroupLayoutEntry {
            binding: uniform.binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        group_entries
            .entry(uniform.group)
            .or_insert(Vec::new())
            .push(entry);
    }

    let mut layouts = HashMap::new();
    for (group, entries) in group_entries {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("Bind Group Layout for group: {}", group)),
            entries: entries.as_slice(),
        });
        layouts.insert(group, layout);
    }

    layouts
}

fn create_render_pipeline(
    device: &wgpu::Device,
    bind_group_layouts: &HashMap<u32, wgpu::BindGroupLayout>,
    shader: &crate::ShaderObject,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let mut vs_entry = None;
    let mut fs_entry = None;
    for entry_point in shader.module.entry_points.iter() {
        if entry_point.stage == naga::ShaderStage::Vertex {
            vs_entry = entry_point.function.name.clone();
        }
        if entry_point.stage == naga::ShaderStage::Fragment {
            fs_entry = entry_point.function.name.clone();
        }
    }

    let module = Cow::Owned(shader.module.clone());
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Naga(module),
    });

    let mut sorted_groups: Vec<_> = bind_group_layouts.keys().collect();
    sorted_groups.sort();
    let bind_group_layouts_sorted: Vec<_> = sorted_groups
        .into_iter()
        .map(|g| bind_group_layouts.get(g).unwrap())
        .collect();

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &bind_group_layouts_sorted,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some(vs_entry.as_deref().unwrap_or("vs_main")),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some(fs_entry.as_deref().unwrap_or("fs_main")),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                // @TODO implement more granular control over blending
                // blend: Some(wgpu::BlendState {
                //     color: wgpu::BlendComponent {
                //         src_factor: wgpu::BlendFactor::One,
                //         dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                //         operation: wgpu::BlendOperation::Add,
                //     },
                //     alpha: wgpu::BlendComponent {
                //         src_factor: wgpu::BlendFactor::One,
                //         dst_factor: wgpu::BlendFactor::Zero,
                //         operation: wgpu::BlendOperation::Add,
                //     },
                // }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
