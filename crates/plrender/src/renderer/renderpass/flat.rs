use crate::{
    components,
    renderer::{
        target::HasSize, Commands, RenderContext, RenderPass, RenderTarget, RenderTargetCollection,
        Renderer,
    },
    scene::Scene,
};
use std::mem;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Locals {
    position: [f32; 4],
    scale: [f32; 4],
    rotation: [f32; 4],
    // x0,y0, x1,y1
    bounds: [f32; 4],
    // u0,v0, u1,v1
    tex_coords: [f32; 4],
}

#[derive(Eq, Hash, PartialEq)]
struct LocalKey {
    uniform_buf_index: usize,
    image: crate::TextureId,
}

struct Pipelines {
    transparent: wgpu::RenderPipeline,
}

struct Instance {
    camera_distance: f32,
    locals_bl: super::BufferLocation,
    image: crate::TextureId,
}

pub struct Flat2D<'r> {
    renderer: &'r Renderer,
    global_uniform_buf: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
    local_bind_group_layout: wgpu::BindGroupLayout,
    local_bind_groups: fxhash::FxHashMap<LocalKey, wgpu::BindGroup>,
    uniform_pool: super::BufferPool,
    pipelines: Pipelines,
    temp: Vec<Instance>,
}

impl<'r> Flat2D<'r> {
    pub fn new(renderer: &'r Renderer) -> Self {
        let device = renderer.device();

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("2D Renderpass: Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("flat.wgsl").into()),
        });

        let globals_size = mem::size_of::<Globals>() as wgpu::BufferAddress;
        let global_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("2D Renderpass: Global Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(globals_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let global_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Renderpass: Global Uniform Buffer"),
            size: globals_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("2D Renderpass: Texture Sampler"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("2D Renderpass: Global Bind Group"),
            layout: &global_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let locals_size = mem::size_of::<Locals>() as wgpu::BufferAddress;
        let local_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("2D Renderpass: Local Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(locals_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D Renderpass: Pipeline Layout"),
            bind_group_layouts: &[&global_bgl, &local_bgl],
            push_constant_ranges: &[],
        });

        let pipelines = {
            let mut sample_count = 1;
            let targets = &renderer
                .targets()
                .all()
                .enumerate()
                .map(|(index, target)| {
                    if index == 0 {
                        sample_count = target.sample_count();
                    }
                    if sample_count != target.sample_count() {
                        log::warn!(
                            "
                            All targets must have the same sample count.
                            The render target {:?} uses {} samples,
                            but the render pass uses {} as
                            defined by the first target.
                            ",
                            target,
                            target.sample_count(),
                            sample_count
                        );
                    }

                    Some(wgpu::ColorTargetState {
                        format: target.format(),
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    })
                })
                .collect::<Vec<Option<wgpu::ColorTargetState>>>();

            let transparent = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("2D Transparent Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    buffers: &[],
                    module: &shader_module,
                    entry_point: "main_vs",
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: sample_count,
                    ..Default::default()
                },
                fragment: Some(wgpu::FragmentState {
                    targets,
                    module: &shader_module,
                    entry_point: "main_fs",
                }),
                multiview: None,
            });

            Pipelines { transparent }
        };

        Self {
            renderer,
            global_uniform_buf,
            global_bind_group,
            local_bind_group_layout: local_bgl,
            local_bind_groups: Default::default(),
            uniform_pool: super::BufferPool::uniform("2D Locals", &device),
            pipelines,
            temp: Vec::new(),
        }
    }
}

impl<'r> RenderPass for Flat2D<'r> {
    fn draw(&mut self, scene: &Scene) -> Result<Commands, wgpu::SurfaceError> {
        let renderer = self.renderer;
        let mut targets = renderer.targets();
        let device = renderer.device();

        // @TODO!
        let camera = scene.camera();

        let nodes = scene.get_global_transforms();
        let cam_node = &nodes[camera.node_id];
        self.uniform_pool.reset();
        let queue = renderer.queue();

        let mut commands = Vec::new();

        for target in targets.all_mut() {
            // a camera must be bound to a target because of the aspect ratio
            {
                let m_proj = camera.projection_matrix(target.aspect());
                let m_view_inv = cam_node.inverse_matrix();
                let m_final = glam::Mat4::from(m_proj) * glam::Mat4::from(m_view_inv);
                let globals = Globals {
                    view_proj: m_final.to_cols_array_2d(),
                };
                queue.write_buffer(&self.global_uniform_buf, 0, bytemuck::bytes_of(&globals));
            }

            self.temp.clear();
            self.uniform_pool.reset();
            let cam_dir = glam::Quat::from_slice(&cam_node.rotation) * -glam::Vec3::Z;

            // gather all sprites
            for (_, sprite) in scene.state().query::<&components::Sprite>().iter() {
                let local = &nodes[sprite.node_id];
                let cam_vector = glam::Vec3::from_slice(&local.position)
                    - glam::Vec3::from_slice(&cam_node.position);
                let camera_distance = cam_vector.dot(cam_dir);

                let resources = renderer.resources();
                let image = resources.get_texture(sprite.image);
                let locals = Locals {
                    position: local.position,
                    scale: local.scale,
                    rotation: local.rotation,
                    bounds: {
                        let (w, h) = match sprite.uv {
                            Some(ref uv) => (uv.end.x - uv.start.x, uv.end.y - uv.start.y),
                            None => (image.size.width as i16, image.size.height as i16),
                        };
                        [
                            -0.5 * w as f32,
                            -0.5 * h as f32,
                            0.5 * w as f32,
                            0.5 * w as f32,
                        ]
                    },
                    tex_coords: match sprite.uv {
                        Some(ref uv) => [
                            uv.start.x as f32 / image.size.width as f32,
                            uv.start.y as f32 / image.size.height as f32,
                            uv.end.x as f32 / image.size.width as f32,
                            uv.end.y as f32 / image.size.height as f32,
                        ],
                        None => [0.0, 0.0, 1.0, 1.0],
                    },
                };
                let locals_bl = self.uniform_pool.alloc(&locals, queue);

                // pre-create local bind group, if needed
                let local_bgl = &self.local_bind_group_layout;
                let key = LocalKey {
                    uniform_buf_index: locals_bl.index,
                    image: sprite.image,
                };
                let binding = self.uniform_pool.binding::<Locals>(locals_bl.index);
                self.local_bind_groups.entry(key).or_insert_with(|| {
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("2D Locals"),
                        layout: local_bgl,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(binding),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&image.view),
                            },
                        ],
                    })
                });

                self.temp.push(Instance {
                    camera_distance,
                    locals_bl,
                    image: sprite.image,
                });
            }

            // sort from back to front
            self.temp
                .sort_by_key(|s| (s.camera_distance * -1000.0) as i64);

            let frame = target.next_frame()?;

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("2D Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(camera.background.into()),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });
                pass.set_pipeline(&self.pipelines.transparent);
                pass.set_bind_group(0, &self.global_bind_group, &[]);

                for inst in self.temp.drain(..) {
                    let key = LocalKey {
                        uniform_buf_index: inst.locals_bl.index,
                        image: inst.image,
                    };
                    let local_bg = &self.local_bind_groups[&key];
                    pass.set_bind_group(1, local_bg, &[inst.locals_bl.offset]);
                    pass.draw(0..4, 0..1);
                }
            }

            commands.append(&mut vec![encoder.finish()]);
            target.prepare_render(renderer, frame, &mut commands);
        }

        Ok(commands)
    }
}
