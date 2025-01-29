use std::collections::HashMap;

use crate::{Shader, Uniform};

pub type Commands = Vec<wgpu::CommandBuffer>;

type Error = Box<dyn std::error::Error>;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// 🎨 Draws things on the screen or on a texture.
///
/// The Renderer is the link between the CPU world and the GPU world.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,

    pipelines: HashMap<String, wgpu::RenderPipeline>,
    bind_groups: HashMap<String, wgpu::BindGroup>,
    buffers: HashMap<String, wgpu::Buffer>,
}

unsafe impl Sync for Renderer {}

impl Renderer {
    /// Creates a new Renderer instance.
    pub async fn new<W: HasDisplayHandle + HasWindowHandle + Sync>(
        window: Option<&W>,
    ) -> Result<Renderer, Error> {
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
            .ok_or("Failed to find an appropriate GPU adapter")?;

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

        Ok(Renderer {
            device,
            queue,
            pipelines: HashMap::new(),
            bind_groups: HashMap::new(),
            buffers: HashMap::new(),
        })
    }

    /// Renders a frame from the given Scene with the given RenderPass
    pub fn render(&self, _frame: &Shader) -> Result<(), wgpu::SurfaceError> {
        // FIXME

        // Records the render commands in the GPU command buffer
        // let (commands, frames) =
        //     renderpass.draw(self, scene, scene.first_camera().unwrap(), self.targets)?;

        // // Runs the commands (submit to GPU queue)
        // self.queue.submit(commands);

        // // Shows the rendered frames on the screen
        // if let Ok(mut targets) = self.write_targets() {
        //     targets.present(frames);
        // } else {
        //     log::warn!("Dropped Frame: Cannot present! Failed to acquire Render Targets Database Write lock.");
        //     return Err(wgpu::SurfaceError::Lost);
        // };

        Ok(())
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
    source: &str,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
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
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
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
