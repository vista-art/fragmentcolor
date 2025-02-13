use crate::buffer_pool::BufferPool;
use crate::Target;
use crate::{shader::Uniform, ComputePass, Pass, RenderPass, Shader, ShaderError};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
pub type Commands = Vec<wgpu::CommandBuffer>;

use crate::error::RendererError;

use winit::window::Window;

pub trait Renderable {
    fn passes(&self) -> impl IntoIterator<Item = &Pass>;
    fn targets(&self) -> impl IntoIterator<Item = &Target>;
}

/// Draws things on the screen or on a texture.
///
/// Owns and manages all GPU resources.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    render_pipelines: RefCell<HashMap<[u8; 64], wgpu::RenderPipeline>>,
    compute_pipelines: HashMap<String, wgpu::ComputePipeline>,

    shaders: HashMap<String, wgpu::ShaderModule>,
    bind_group_layouts: HashMap<String, wgpu::BindGroupLayout>,
    bind_groups: HashMap<String, wgpu::BindGroup>,
    buffer_pool: HashMap<String, BufferPool>,

    textures: HashMap<String, wgpu::Texture>,
    samplers: HashMap<String, wgpu::Sampler>,
}

unsafe impl Sync for Renderer {}

pub async fn init_offfscreen() -> Result<Renderer, RendererError> {
    init_renderer(None).await
}

// TODO: Implement platform-specific initializers
pub async fn init_renderer(window: Option<&Window>) -> Result<Renderer, RendererError> {
    let instance = wgpu::Instance::default();

    let surface = if let Some(window) = window {
        instance.create_surface(window).ok()
    } else {
        None
    };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: surface.as_ref(),
        })
        .await
        .ok_or(RendererError::AdapterError)?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None, // Trace path
        )
        .await?;

    device.on_uncaptured_error(Box::new(|error| {
        log::error!("\n\n==== GPU error: ====\n\n{:#?}\n", error);
    }));

    Ok(Renderer::new(device, queue))
}

impl Renderer {
    /// Creates a new Renderer instance.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Renderer {
        Renderer {
            device,
            queue,

            render_pipelines: RefCell::new(HashMap::new()),
            compute_pipelines: HashMap::new(),

            shaders: HashMap::new(),
            bind_group_layouts: HashMap::new(),

            bind_groups: HashMap::new(),
            textures: HashMap::new(),
            buffer_pool: HashMap::new(),
            samplers: HashMap::new(),
        }
    }

    /// Renders a frame
    pub fn render(&self, renderable: &impl Renderable) -> Result<(), ShaderError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut presentations = Vec::new();
        for target in renderable.targets() {
            let presentation = target.get_current_texture()?;
            presentations.push(presentation);
        }

        for pass in renderable.passes() {
            match &*pass {
                Pass::Render(render_pass) => self.process_render_pass(&mut encoder, render_pass)?,
                Pass::Compute(compute_pass) => {
                    self.process_compute_pass(&mut encoder, compute_pass)?
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));

        for presentation in presentations {
            presentation.present();
        }

        Ok(())
    }

    fn process_render_pass(
        &self,
        _encoder: &mut wgpu::CommandEncoder,
        pass: &RenderPass,
    ) -> Result<(), ShaderError> {
        for shader in &pass.shaders {
            self.ensure_render_pipeline(shader)?;
        }

        println!("Processing render pass: {:#?}", pass);

        // @TODO

        Ok(())
    }

    fn process_compute_pass(
        &self,
        _encoder: &mut wgpu::CommandEncoder,
        _pass: &ComputePass,
    ) -> Result<(), ShaderError> {
        Ok(()) // @TODO
    }

    fn ensure_render_pipeline(&self, shader: &Shader) -> Result<(), ShaderError> {
        let mut pipelines = self.render_pipelines.borrow_mut();
        if pipelines.contains_key(&shader.hash()) {
            return Ok(());
        }

        let bgl = create_bind_group_layouts(&self.device, &shader.uniforms);

        let module = Cow::Owned(shader.module.clone()); // @TODO can we avoid the clone?
        let pipeline = create_render_pipeline(&self.device, &bgl, module);

        pipelines.insert(shader.hash.clone(), pipeline);
        Ok(())
    }

    // @TODO [u8; 64] should be a wrapped type
    /// Provides a way to invalidate the cache; i.e. when a Shader source changes
    pub(crate) fn remove_render_pipeline(&mut self, key: [u8; 64]) {
        self.render_pipelines.borrow_mut().remove(&key);
    }
}

fn create_bind_group_layouts(
    device: &wgpu::Device,
    uniforms: &HashMap<String, Uniform>,
) -> HashMap<u32, wgpu::BindGroupLayout> {
    let mut layouts = HashMap::new();
    for uniform in uniforms.values() {
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
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("BindGroupLayout"),
            entries: &[entry],
        });
        layouts.insert(uniform.group, layout);
    }
    layouts
}

fn create_render_pipeline(
    device: &wgpu::Device,
    bind_group_layouts: &HashMap<u32, wgpu::BindGroupLayout>,
    module: Cow<'static, naga::Module>,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Naga(module),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &bind_group_layouts.values().collect::<Vec<_>>(),
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"), // @TODO this must be dynamic
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"), // @TODO this must be dynamic
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8Unorm,
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
