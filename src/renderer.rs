use crate::buffer_pool::BufferPool;
use crate::TargetFrame;
use crate::{
    shader::Uniform, ComputePass, Pass, RenderPass, Shader, ShaderError, ShaderHash, Target,
};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
pub type Commands = Vec<wgpu::CommandBuffer>;

pub trait Renderable {
    fn passes(&self) -> impl IntoIterator<Item = &Pass>;
}

struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layouts: Vec<wgpu::BindGroupLayout>,
}

/// Draws things on the screen or a texture.
///
/// It owns and manages all GPU resources, serving as the
/// main graphics context provider for the application.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    render_pipelines: RefCell<HashMap<ShaderHash, RenderPipeline>>,
    _compute_pipelines: RefCell<HashMap<String, wgpu::ComputePipeline>>,

    _shaders: RefCell<HashMap<String, wgpu::ShaderModule>>,
    _bind_groups: RefCell<HashMap<String, wgpu::BindGroup>>,

    _textures: RefCell<HashMap<String, wgpu::Texture>>,
    _samplers: RefCell<HashMap<String, wgpu::Sampler>>,

    buffer_pool: RefCell<BufferPool>,
}

impl Renderer {
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

impl Renderer {
    /// Creates a new Renderer instance.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Renderer {
        let buffer_pool = BufferPool::new_uniform_pool("Uniform Buffer Pool", &device);

        Renderer {
            device,
            queue,

            render_pipelines: RefCell::new(HashMap::new()),
            _compute_pipelines: RefCell::new(HashMap::new()),

            _shaders: RefCell::new(HashMap::new()),
            _bind_groups: RefCell::new(HashMap::new()),

            _textures: RefCell::new(HashMap::new()),
            _samplers: RefCell::new(HashMap::new()),

            buffer_pool: RefCell::new(buffer_pool),
        }
    }

    /// Renders a Frame or Shader to a Target.
    pub fn render(
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
            match pass {
                Pass::Render(render_pass) => {
                    self.process_render_pass(&mut encoder, render_pass, frame.as_ref())?
                }
                Pass::Compute(compute_pass) => {
                    self.process_compute_pass(&mut encoder, compute_pass)?
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));

        frame.present();

        Ok(())
    }
}

impl Renderer {
    fn process_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pass: &RenderPass,
        frame: &dyn TargetFrame,
    ) -> Result<(), ShaderError> {
        self.buffer_pool.borrow_mut().reset();

        let attachments = &[Some(wgpu::RenderPassColorAttachment {
            view: frame.view(),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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

        for shader in &pass.shaders {
            self.ensure_render_pipeline(shader)?;
            let pipelines = self.render_pipelines.borrow();
            let cached = pipelines.get(&shader.hash).unwrap();

            let storage_size = shader.storage().uniform_bytes.len() * std::mem::size_of::<u8>();
            self.buffer_pool
                .borrow_mut()
                .ensure_capacity(storage_size as u64, &self.device);

            let mut bind_group_entries: HashMap<u32, Vec<wgpu::BindGroupEntry>> = HashMap::new();

            let mut buffer_locations = Vec::new();
            // @TODO there should be a method get_bind_groups() in the Shader
            for name in &shader.list_uniforms() {
                if name.contains('.') {
                    continue;
                }

                let uniform = shader.get_uniform(name).unwrap();

                // @FIXME TECH DEBT: this is wrong. We are looping through an internal data structure
                // that should be hidden, and then using a public method to access the same structure.
                // I need to figure out a better way to handle this.
                let storage = shader.storage();
                let bytes = storage
                    .get_bytes(name)
                    .ok_or(ShaderError::UniformNotFound(name.clone()))?;

                let buffer_location = {
                    let mut buffer_pool = self.buffer_pool.borrow_mut();
                    buffer_pool.upload(bytes, &self.queue)
                };

                buffer_locations.push((uniform, buffer_location));
            }

            let buffer_pool = self.buffer_pool.borrow();
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
            render_pass.draw(0..3, 0..1); // Fullscreen triangle
        }
        Ok(())
    }

    fn process_compute_pass(
        &self,
        _encoder: &mut wgpu::CommandEncoder,
        _pass: &ComputePass,
    ) -> Result<(), ShaderError> {
        Ok(()) // @TODO later
    }

    fn ensure_render_pipeline(&self, shader: &Shader) -> Result<(), ShaderError> {
        let mut pipelines = self.render_pipelines.borrow_mut();

        pipelines.entry(shader.hash).or_insert_with(|| {
            let layouts = create_bind_group_layouts(&self.device, &shader.storage().uniforms);
            let pipeline = create_render_pipeline(&self.device, &layouts, &shader.module);

            RenderPipeline {
                pipeline,
                bind_group_layouts: layouts.values().cloned().collect(),
            }
        });

        Ok(())
    }
}

// @TODO avoid tight coupling with storage internals
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
    module: &naga::Module,
) -> wgpu::RenderPipeline {
    let mut vs_entry = None;
    let mut fs_entry = None;
    for entry_point in module.entry_points.iter() {
        if entry_point.stage == naga::ShaderStage::Vertex {
            vs_entry = entry_point.function.name.clone();
        }
        if entry_point.stage == naga::ShaderStage::Fragment {
            fs_entry = entry_point.function.name.clone();
        }
    }

    let module = Cow::Owned(module.clone());
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
                format: wgpu::TextureFormat::Bgra8Unorm, // @TODO dynamic format
                blend: Some(wgpu::BlendState::REPLACE),
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
