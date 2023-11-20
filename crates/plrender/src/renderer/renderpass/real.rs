use crate::{
    math::geometry::{Normal, Position, TextureCoordinates, Vertex},
    renderer::{
        target::Dimensions, RenderContext, RenderPassResult, RenderTarget, RenderTargetCollection,
        Renderer,
    },
    resources::{mesh::MeshId, texture::TextureId},
    scene::SceneState,
};
use fxhash::FxHashMap;
use std::{mem, sync::RwLockReadGuard};
use wgpu::util::DeviceExt as _;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub base_color_map: Option<crate::TextureId>,
    pub emissive_color: crate::Color,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color_map: None,
            emissive_color: crate::Color(0),
            metallic_factor: 1.0,
            roughness_factor: 0.0,
            normal_scale: 1.0,
            occlusion_strength: 1.0,
        }
    }
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
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
    base_color_factor: [f32; 4],
    emissive_factor: [f32; 4],
    metallic_roughness_values: [f32; 2],
    normal_scale: f32,
    occlusion_strength: f32,
}

#[derive(Eq, Hash, PartialEq)]
struct LocalKey {
    uniform_buf_index: usize,
    base_color_map: Option<crate::TextureId>,
}

#[derive(Debug)]
pub struct RealConfig {
    pub cull_back_faces: bool,
    pub max_lights: usize,
}

impl Default for RealConfig {
    fn default() -> Self {
        Self {
            cull_back_faces: true,
            max_lights: 16,
        }
    }
}

struct Pipelines {
    main: wgpu::RenderPipeline,
}

struct Instance {
    mesh_id: MeshId,
    locals_bl: super::BufferLocation,
    base_color_map: Option<TextureId>,
}

/// Realistic renderer.
/// Follows Disney PBR.
pub struct Real<'r> {
    renderer: &'r Renderer,
    depth_texture: Option<(wgpu::TextureView, wgpu::Extent3d)>,
    global_uniform_buf: wgpu::Buffer,
    light_buf: wgpu::Buffer,
    light_capacity: usize,
    global_bind_group: wgpu::BindGroup,
    local_bind_group_layout: wgpu::BindGroupLayout,
    local_bind_groups: FxHashMap<LocalKey, wgpu::BindGroup>,
    uniform_pool: super::BufferPool,
    pipelines: Pipelines,
    blank_color_view: wgpu::TextureView,
    instances: Vec<Instance>,
}

impl<'r> Real<'r> {
    pub fn new(config: &RealConfig, renderer: &'r Renderer) -> Self {
        let d = renderer.device();
        let shader_module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("real"),
            source: wgpu::ShaderSource::Wgsl(include_str!("real.wgsl").into()),
        });

        let globals_size = mem::size_of::<Globals>() as wgpu::BufferAddress;
        let global_bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("real globals"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<Light>() as wgpu::BufferAddress
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let global_uniform_buf = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("real globals"),
            size: globals_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buf = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("real lights"),
            size: (config.max_lights * mem::size_of::<Light>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = d.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("real sampler"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let global_bind_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("real globals"),
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let locals_size = mem::size_of::<Locals>() as wgpu::BufferAddress;
        let local_bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("real locals"),
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

        let pipelines = {
            let pipeline_layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("real"),
                bind_group_layouts: &[&global_bgl, &local_bgl],
                push_constant_ranges: &[],
            });
            let primitive = wgpu::PrimitiveState {
                cull_mode: if config.cull_back_faces {
                    Some(wgpu::Face::Back)
                } else {
                    None
                },
                ..Default::default()
            };

            let mut sample_count = 1;
            let targets = &renderer
                .read_targets()
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
                main: d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("real"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        buffers: &[
                            Position::layout::<0>(),
                            TextureCoordinates::layout::<1>(),
                            Normal::layout::<2>(),
                        ],
                        module: &shader_module,
                        entry_point: "main_vs",
                    },
                    primitive,
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEPTH_FORMAT,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        depth_write_enabled: true,
                        bias: Default::default(),
                        stencil: Default::default(),
                    }),
                    multisample,
                    fragment: Some(wgpu::FragmentState {
                        targets,
                        module: &shader_module,
                        entry_point: "main_fs",
                    }),
                    multiview: None,
                }),
            }
        };

        let blank_color_view = {
            let desc = wgpu::TextureDescriptor {
                label: Some("dummy"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
            };
            let texture = d.create_texture_with_data(renderer.queue(), &desc, &[0xFF; 4]);
            texture.create_view(&wgpu::TextureViewDescriptor::default())
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
            uniform_pool: super::BufferPool::uniform("real locals", d),
            pipelines,
            blank_color_view,
            instances: Vec::new(),
        }
    }
}

impl<'r> crate::RenderPass for Real<'r> {
    fn draw(&mut self, scene: RwLockReadGuard<'_, SceneState>) -> RenderPassResult {
        let renderer = self.renderer;
        let targets = renderer.read_targets();
        let resources = renderer.read_resources();
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

                // @TODO those things should be cached; not calculated every frame
                //       invalidate on screen resize
                if !camera_target.clip_region.equals(target.size()) {
                    //if camera_target.clip_region.is_smaller_than(target.size().to_wgpu_size()) {

                    // I don't expect to enter here.
                    //
                    // This is not a priority, but the current logic won't work
                    // for multiple viewports in the same window.
                    //
                    // @TODO is this target targeted by multiple cameras?
                    //
                    //       - if this is the only camera rendering to it fullscreen,
                    //         we'll proceed as usual. (this is the expected use case for now).
                    //
                    //       - if this is not the only camera, we need to cache the results
                    //         and start a new pass with LoadOp::Load instead of "Clear"
                    log::warn!(
                        "Camera {:?} is targeting a target {:?} with a different size.",
                        object_id,
                        camera_target.target_id
                    );
                }

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

                let nodes = scene.get_global_transforms();
                self.uniform_pool.reset();
                let queue = renderer.queue();

                {
                    let m_proj = camera.projection_matrix(target.aspect());
                    let node = &nodes[camera.node_id];
                    let m_view_inv = node.inverse_matrix();
                    let m_final = glam::Mat4::from(m_proj) * glam::Mat4::from(m_view_inv);
                    let globals = Globals {
                        view_proj: m_final.to_cols_array_2d(),
                        camera_pos: node.position,
                    };
                    queue.write_buffer(&self.global_uniform_buf, 0, bytemuck::bytes_of(&globals));
                }

                let lights = scene
                    .query::<&crate::Light>()
                    .iter()
                    .map(|(_, light)| {
                        let space = &nodes[light.node_id];
                        let mut position = space.position;
                        position[3] = match light.variant {
                            crate::LightType::Directional => 0.0,
                            crate::LightType::Point => 1.0,
                        };
                        let mut color_intensity = light.color.into_vec4();
                        color_intensity[3] = light.intensity;
                        Light {
                            position,
                            rotation: space.rotation,
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

                //TODO: we can do everything in a single pass if we use some arena-based hashmap.
                self.instances.clear();

                for (_, (entity, &color, mat)) in scene
                    .query::<(&crate::Mesh, &crate::Color, &Material)>()
                    .with::<&Vertex<Position>>()
                    .with::<&Vertex<TextureCoordinates>>()
                    .with::<&Vertex<Normal>>()
                    .iter()
                {
                    let space = &nodes[entity.node_id];

                    let locals = Locals {
                        position: space.position,
                        scale: space.scale,
                        rotation: space.rotation,
                        base_color_factor: color.into_vec4(),
                        emissive_factor: mat.emissive_color.into_vec4(),
                        metallic_roughness_values: [mat.metallic_factor, mat.roughness_factor],
                        normal_scale: mat.normal_scale,
                        occlusion_strength: mat.occlusion_strength,
                    };
                    let locals_bl = self.uniform_pool.alloc(&locals, queue);

                    // pre-create local bind group, if needed
                    let key = LocalKey {
                        uniform_buf_index: locals_bl.index,
                        base_color_map: mat.base_color_map,
                    };
                    let binding = self.uniform_pool.binding::<Locals>(locals_bl.index);
                    let local_bgl = &self.local_bind_group_layout;
                    let blank_color_view = &self.blank_color_view;

                    self.local_bind_groups.entry(key).or_insert_with(|| {
                        let base_color_view = match mat.base_color_map {
                            Some(texture) => {
                                if let Some(texture) = resources.get_texture(&texture) {
                                    &texture.view
                                } else {
                                    blank_color_view
                                }
                            }
                            None => blank_color_view,
                        };
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("real locals"),
                            layout: local_bgl,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::Buffer(binding),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(&base_color_view),
                                },
                            ],
                        })
                    });

                    self.instances.push(Instance {
                        mesh_id: entity.mesh_id,
                        locals_bl,
                        base_color_map: mat.base_color_map,
                    });
                }

                let frame = target.next_frame()?;

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("real"),
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

                    pass.set_pipeline(&self.pipelines.main);
                    pass.set_bind_group(0, &self.global_bind_group, &[]);

                    for instance in self.instances.drain(..) {
                        let mesh = if let Some(mesh) = resources.get_mesh(&instance.mesh_id) {
                            mesh
                        } else {
                            continue;
                        };

                        let key = LocalKey {
                            uniform_buf_index: instance.locals_bl.index,
                            base_color_map: instance.base_color_map,
                        };
                        let local_bg = &self.local_bind_groups[&key];
                        pass.set_bind_group(1, local_bg, &[instance.locals_bl.offset]);

                        pass.set_vertex_buffer(0, mesh.vertex_slice::<Position>());
                        pass.set_vertex_buffer(1, mesh.vertex_slice::<TextureCoordinates>());
                        pass.set_vertex_buffer(2, mesh.vertex_slice::<Normal>());

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
