use crate::{
    components,
    math::geometry::vertex,
    math::geometry::vertex::{Normal, Position, Vertex},
    renderer::{
        renderpass::buffer, target::Dimensions, IsRenderTarget, RenderContext, RenderPass,
        RenderPassResult, RenderTargetCollection, Renderer,
    },
    scene::SceneState,
    Color,
};
use fxhash::FxHashMap;
use std::{mem, sync::RwLockReadGuard};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ShaderType {
    Gouraud { flat: bool },
    Phong { glossiness: u8 },
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
const INTENSITY_THRESHOLD: f32 = 0.1;
const LIGHT_COUNT: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    ambient: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Light {
    position: [f32; 4],
    rotation: [f32; 4],
    color_intensity: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Locals {
    position: [f32; 4],
    scale: [f32; 4],
    rotation: [f32; 4],
    color: [f32; 4],
    lights: [u32; LIGHT_COUNT],
    glossiness: f32,
    _pad: [f32; 3],
}

struct Pipelines {
    flat: wgpu::RenderPipeline,
    gouraud: wgpu::RenderPipeline,
    phong: wgpu::RenderPipeline,
}

#[derive(Eq, Hash, PartialEq)]
struct LocalKey {
    uniform_buf_index: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Ambient {
    pub color: Color,
    pub intensity: f32,
}

impl Default for Ambient {
    fn default() -> Self {
        Self {
            color: Color(0xFFFFFFFF),
            intensity: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct PhongConfig {
    pub cull_back_faces: bool,
    pub ambient: Ambient,
    pub max_lights: usize,
}

impl Default for PhongConfig {
    fn default() -> Self {
        Self {
            cull_back_faces: true,
            ambient: Ambient::default(),
            max_lights: 16,
        }
    }
}

pub(crate) struct Phong<'r> {
    renderer: &'r Renderer,
    depth_texture: Option<(wgpu::TextureView, wgpu::Extent3d)>,
    global_uniform_buf: wgpu::Buffer,
    light_buf: wgpu::Buffer,
    light_capacity: usize,
    global_bind_group: wgpu::BindGroup,
    local_bind_group_layout: wgpu::BindGroupLayout,
    local_bind_groups: FxHashMap<LocalKey, wgpu::BindGroup>,
    uniform_pool: buffer::BufferPool,
    pipelines: Pipelines,
    ambient: Ambient,
    temp_lights: Vec<(f32, u32)>,
}

impl<'r> Phong<'r> {
    #[allow(dead_code)]
    pub(crate) fn new(config: &PhongConfig, renderer: &'r Renderer) -> Self {
        let d = renderer.device();

        let shader_module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("phong"),
            source: wgpu::ShaderSource::Wgsl(include_str!("phong.wgsl").into()),
        });

        let globals_size = mem::size_of::<Globals>() as wgpu::BufferAddress;
        let global_bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("phong globals"),
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
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<Light>() as wgpu::BufferAddress
                        ),
                    },
                    count: None,
                },
            ],
        });
        let global_uniform_buf = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phong globals"),
            size: globals_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buf = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phong lights"),
            size: (config.max_lights * mem::size_of::<Light>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let global_bind_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("phong globals"),
            layout: &global_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buf.as_entire_binding(),
                },
            ],
        });

        let locals_size = mem::size_of::<Locals>() as wgpu::BufferAddress;
        let local_bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("phong locals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(locals_size),
                },
                count: None,
            }],
        });

        let pipelines = {
            let pipeline_layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("phong"),
                bind_group_layouts: &[&global_bgl, &local_bgl],
                push_constant_ranges: &[],
            });
            let vertex_buffers = [
                vertex::Position::layout::<0>(),
                vertex::Normal::layout::<1>(),
            ];
            let primitive = wgpu::PrimitiveState {
                cull_mode: if config.cull_back_faces {
                    Some(wgpu::Face::Back)
                } else {
                    None
                },
                ..Default::default()
            };
            let ds = Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                bias: Default::default(),
                stencil: Default::default(),
            });

            let mut sample_count = 1;
            let targets = renderer
                .read_targets()
                .expect("lock poisoned")
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
                            but the first target pass uses {}.
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

            let multisample = wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
            };

            Pipelines {
                flat: d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("phong/flat"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        buffers: &vertex_buffers,
                        module: &shader_module,
                        entry_point: "vs_flat",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    primitive,
                    depth_stencil: ds.clone(),
                    multisample,
                    fragment: Some(wgpu::FragmentState {
                        targets: targets.as_ref(),
                        module: &shader_module,
                        entry_point: "fs_flat",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview: None,
                }),
                gouraud: d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("phong/gouraud"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        buffers: &vertex_buffers,
                        module: &shader_module,
                        entry_point: "vs_flat",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    primitive,
                    depth_stencil: ds.clone(),
                    multisample,
                    fragment: Some(wgpu::FragmentState {
                        targets: targets.as_ref(),
                        module: &shader_module,
                        entry_point: "fs_gouraud",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview: None,
                }),
                phong: d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("phong"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        buffers: &vertex_buffers,
                        module: &shader_module,
                        entry_point: "vs_phong",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    primitive,
                    depth_stencil: ds.clone(),
                    multisample,
                    fragment: Some(wgpu::FragmentState {
                        targets: targets.as_ref(),
                        module: &shader_module,
                        entry_point: "fs_phong",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    multiview: None,
                }),
            }
        };

        Self {
            renderer,
            depth_texture: None,
            global_uniform_buf,
            light_capacity: config.max_lights,
            light_buf,
            global_bind_group,
            local_bind_group_layout: local_bgl,
            local_bind_groups: Default::default(),
            uniform_pool: buffer::BufferPool::uniform("phong locals", d),
            pipelines,
            ambient: config.ambient,
            temp_lights: Vec::new(),
        }
    }
}

impl<'r> RenderPass for Phong<'r> {
    fn draw(&mut self, scene: RwLockReadGuard<'_, SceneState>) -> RenderPassResult {
        let renderer = self.renderer;
        let targets = renderer.read_targets().expect("lock poisoned");
        let device = renderer.device();

        let mut commands = Vec::new();

        let mut rendered_frames = Vec::new();
        for (object_id, camera) in scene.cameras().iter() {
            let camera_targets = scene.get_camera_targets(object_id);

            // NOTE: Don't mind the nested loop.
            // Most of the times, there will be only one target and one camera.
            for camera_target in camera_targets {
                let target = targets.get(&camera_target.target_id);

                if target.is_none() {
                    log::error!(
                        "Camera {:?} is targeting a non-existent target {:?}",
                        object_id,
                        camera_target.target_id
                    );
                    continue;
                };

                let target = target.unwrap();

                let reset_depth = match self.depth_texture {
                    Some((_, size)) => size != target.size().to_wgpu_size(),
                    None => true,
                };
                if reset_depth {
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("depth"),
                        dimension: wgpu::TextureDimension::D2,
                        format: DEPTH_FORMAT,
                        size: target.size().to_wgpu_size(),
                        sample_count: 1,
                        mip_level_count: 1,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[DEPTH_FORMAT],
                    });
                    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                    self.depth_texture = Some((view, target.size().to_wgpu_size()));
                }

                let transforms = scene.calculate_global_transforms();
                self.uniform_pool.reset();
                let queue = renderer.queue();

                {
                    let m_proj = camera.projection_matrix(target.aspect());
                    let m_view_inv = transforms[camera.transform_id].inverse_matrix();
                    let m_final = m_proj * glam::Mat4::from(m_view_inv);
                    let ambient = self.ambient.color.to_array();
                    let globals = Globals {
                        view_proj: m_final.to_cols_array_2d(),
                        ambient: [
                            ambient[0] * self.ambient.intensity,
                            ambient[1] * self.ambient.intensity,
                            ambient[2] * self.ambient.intensity,
                            0.0,
                        ],
                    };
                    queue.write_buffer(&self.global_uniform_buf, 0, bytemuck::bytes_of(&globals));
                }

                let lights = scene
                    .query::<&components::Light>()
                    .iter()
                    .map(|(_, light)| {
                        let transform = &transforms[light.transform_id];
                        let mut position = transform.position;
                        position[3] = match light.variant {
                            components::LightType::Directional => 0.0,
                            components::LightType::Point => 1.0,
                        };
                        let mut color_intensity = light.color.to_array();
                        color_intensity[3] = light.intensity;
                        Light {
                            position,
                            rotation: transform.rotation,
                            color_intensity,
                        }
                    })
                    .collect::<Vec<_>>();

                let light_count = lights.len().min(self.light_capacity);
                queue.write_buffer(
                    &self.light_buf,
                    0,
                    bytemuck::cast_slice(&lights[..light_count]),
                );

                // pre-create the bind groups so that we don't need to do it on the fly
                let local_bgl = &self.local_bind_group_layout;
                let entity_count = scene
                    .query::<(&components::Mesh, &components::Color, &ShaderType)>()
                    .with::<&Vertex<Position>>()
                    .with::<&Vertex<Normal>>()
                    .iter()
                    .count();
                let uniform_pool_size = self
                    .uniform_pool
                    .prepare_for_count::<Locals>(entity_count, device);
                for uniform_buf_index in 0..uniform_pool_size {
                    let key = LocalKey { uniform_buf_index };
                    let binding = self.uniform_pool.binding::<Locals>(uniform_buf_index);

                    self.local_bind_groups.entry(key).or_insert_with(|| {
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("phong locals"),
                            layout: local_bgl,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(binding),
                            }],
                        })
                    });
                }

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let frame = target.next_frame()?;

                let resources = renderer.read_resources().expect("lock poisoned");

                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("phong"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(camera_target.clear_color.into()),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_texture.as_ref().unwrap().0,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        ..Default::default()
                    });

                    pass.set_bind_group(0, &self.global_bind_group, &[]);

                    for (_, (entity, &color, &shader)) in scene
                        .query::<(&components::Mesh, &components::Color, &ShaderType)>()
                        .with::<&Vertex<Position>>()
                        .with::<&Vertex<Normal>>()
                        .iter()
                    {
                        let local = &transforms[entity.transform_id];
                        let mesh = if let Some(mesh) = resources.get_mesh(&entity.mesh_id) {
                            mesh
                        } else {
                            continue;
                        };
                        let entity_radius =
                            mesh.bound_radius * local.scale.into_iter().reduce(f32::max).unwrap();

                        // collect the `LIGHT_COUNT` lights most affecting the entity
                        self.temp_lights.clear();
                        let entity_pos = glam::Vec3::from_slice(&local.position[..3]);
                        let mut lights = scene.query::<&components::Light>();

                        for (index, (_, light)) in lights.iter().enumerate() {
                            let light_pos = glam::Vec3::from_slice(
                                &transforms[light.transform_id].position[..3],
                            );
                            let intensity = match light.variant {
                                components::LightType::Point => {
                                    let distance = (entity_pos - light_pos).length();
                                    if distance <= entity_radius {
                                        light.intensity
                                    } else {
                                        let bound_distance = (distance - entity_radius).max(1.0);
                                        light.intensity / bound_distance * bound_distance
                                    }
                                }
                                components::LightType::Directional => light.intensity,
                            };
                            if intensity > INTENSITY_THRESHOLD {
                                self.temp_lights.push((intensity, index as u32));
                            }
                        }
                        self.temp_lights
                            .sort_by_key(|&(intensity, _)| (1.0 / intensity) as usize);
                        let mut light_indices = [0u32; LIGHT_COUNT];
                        for (li, &(_, index)) in light_indices.iter_mut().zip(&self.temp_lights) {
                            *li = index;
                        }

                        //TODO: check for texture coordinates
                        pass.set_pipeline(match shader {
                            ShaderType::Gouraud { flat: true } => &self.pipelines.flat,
                            ShaderType::Gouraud { flat: false } => &self.pipelines.gouraud,
                            ShaderType::Phong { .. } => &self.pipelines.phong,
                        });

                        let locals = Locals {
                            position: local.position,
                            scale: local.scale,
                            rotation: local.rotation,
                            color: color.into_vec4_gamma(),
                            lights: light_indices,
                            glossiness: match shader {
                                ShaderType::Phong { glossiness } => glossiness as f32,
                                _ => 0.0,
                            },
                            _pad: [0.0; 3],
                        };
                        let bl = self.uniform_pool.alloc(&locals, queue);

                        let key = LocalKey {
                            uniform_buf_index: bl.index,
                        };
                        let local_bg = &self.local_bind_groups[&key];
                        pass.set_bind_group(1, local_bg, &[bl.offset]);

                        pass.set_vertex_buffer(0, mesh.vertex_slice::<vertex::Position>());
                        pass.set_vertex_buffer(1, mesh.vertex_slice::<vertex::Normal>());

                        if let Some(ref is) = mesh.vertex_ids {
                            pass.set_index_buffer(mesh.buffer.slice(is.offset..), is.format);
                            pass.draw_indexed(0..is.count, 0, 0..1);
                        } else {
                            pass.draw(0..mesh.vertex_count, 0..1);
                        }
                    }
                }

                commands.append(&mut vec![encoder.finish()]);
                target.prepare_render(renderer, &mut commands);

                rendered_frames.push((target.id(), frame));
            }
        }

        Ok((commands, rendered_frames))
    }
}
