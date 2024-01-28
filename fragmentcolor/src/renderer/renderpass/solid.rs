use crate::{
    components,
    math::geometry::{Position, Vertex},
    renderer::{
        renderpass::buffer,
        target::{Dimensions, IsRenderTarget, RenderTargetCollection},
        RenderContext, RenderPass, RenderPassResult, Renderer,
    },
    scene::SceneState,
    Color,
};
use bytemuck::{Pod, Zeroable};
use fxhash::FxHashMap;
use std::{mem, sync::RwLockReadGuard};

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
struct Locals {
    position: [f32; 4],
    scale: [f32; 4],
    rotation: [f32; 4],
    color: [f32; 4],
}

#[derive(Eq, Hash, PartialEq)]
struct LocalKey {
    uniform_buf_index: usize,
}

#[derive(Debug)]
pub struct SolidConfig {
    pub cull_back_faces: bool,
}

impl Default for SolidConfig {
    fn default() -> Self {
        Self {
            cull_back_faces: true,
        }
    }
}

pub(crate) struct Solid<'r> {
    renderer: &'r Renderer,
    depth_texture: Option<(wgpu::TextureView, wgpu::Extent3d)>,
    global_uniform_buf: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
    local_bind_group_layout: wgpu::BindGroupLayout,
    local_bind_groups: FxHashMap<LocalKey, wgpu::BindGroup>,
    uniform_pool: buffer::BufferPool,
    pipeline: wgpu::RenderPipeline,
}

impl<'r> Solid<'r> {
    pub(crate) fn new(config: &SolidConfig, renderer: &'r Renderer) -> Self {
        let d = renderer.device();
        let shader_module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("solid"),
            source: wgpu::ShaderSource::Wgsl(include_str!("solid.wgsl").into()),
        });

        let globals_size = mem::size_of::<Globals>() as wgpu::BufferAddress;
        let global_bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("solid globals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(globals_size),
                },
                count: None,
            }],
        });
        let global_uniform_buf = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("solid globals"),
            size: globals_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let global_bind_group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("solid globals"),
            layout: &global_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform_buf.as_entire_binding(),
            }],
        });

        let locals_size = mem::size_of::<Locals>() as wgpu::BufferAddress;
        let local_bgl = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("solid locals"),
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

        let pipeline_layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("solid"),
            bind_group_layouts: &[&global_bgl, &local_bgl],
            push_constant_ranges: &[],
        });

        let targets = &renderer
            .read_targets()
            .expect("read lock poisoned")
            .all()
            .map(|target| {
                Some(wgpu::ColorTargetState {
                    format: target.format(),
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })
            })
            .collect::<Vec<Option<wgpu::ColorTargetState>>>();

        let pipeline = d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("solid"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                buffers: &[Position::layout::<0>()],
                module: &shader_module,
                entry_point: "main_vs",
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: if config.cull_back_faces {
                    Some(wgpu::Face::Back)
                } else {
                    None
                },
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_compare: wgpu::CompareFunction::LessEqual,
                depth_write_enabled: true,
                bias: Default::default(),
                stencil: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                targets,
                module: &shader_module,
                entry_point: "main_fs",
            }),
            multiview: None,
        });

        Self {
            renderer,
            depth_texture: None,
            global_uniform_buf,
            global_bind_group,
            local_bind_group_layout: local_bgl,
            local_bind_groups: Default::default(),
            uniform_pool: buffer::BufferPool::uniform("solid locals", d),
            pipeline,
        }
    }
}

impl<'r> RenderPass for Solid<'r> {
    fn draw(&mut self, scene: RwLockReadGuard<'_, SceneState>) -> RenderPassResult {
        let renderer = self.renderer;
        let device = renderer.device();
        let resources = renderer.read_resources().expect("read lock poisoned");
        let targets = renderer.read_targets().expect("read lock poisoned");

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
                    // @TODO this should not happen here. Our Texture
                    //       implementation contains a method to do this.
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
                    let globals = Globals {
                        view_proj: m_final.to_cols_array_2d(),
                    };
                    queue.write_buffer(&self.global_uniform_buf, 0, bytemuck::bytes_of(&globals));
                }

                // pre-create the bind groups so that we don't need to do it on the fly
                let local_bgl = &self.local_bind_group_layout;

                let entity_count = scene
                    .query::<(&components::Mesh, &Color)>()
                    .with::<&Vertex<Position>>()
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
                            label: Some("solid locals"),
                            layout: local_bgl,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(binding),
                            }],
                        })
                    });
                }

                let frame = target.next_frame()?;

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("solid"),

                        // @TODO loop all targets and add them as views simultaneously
                        //       OPEN QUESTION: must them all be of the same size?
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &frame.view, // <- here
                            resolve_target: None,
                            ops: wgpu::Operations {
                                // @TODO this should be a property of the target,
                                //       instead of the camera.
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
                    pass.set_pipeline(&self.pipeline);
                    pass.set_bind_group(0, &self.global_bind_group, &[]);

                    for (_, (entity, color)) in scene
                        .query::<(&crate::Mesh, &crate::Color)>()
                        .with::<&Vertex<Position>>()
                        .iter()
                    {
                        let local = &transforms[entity.transform_id];
                        let locals = Locals {
                            position: local.position,
                            scale: local.scale,
                            rotation: local.rotation,
                            color: color.into_vec4_gamma(),
                        };
                        let bl = self.uniform_pool.alloc(&locals, queue);

                        let key = LocalKey {
                            uniform_buf_index: bl.index,
                        };
                        let local_bg = &self.local_bind_groups[&key];
                        pass.set_bind_group(1, local_bg, &[bl.offset]);

                        let mesh = if let Some(mesh) = resources.get_mesh(&entity.mesh_id) {
                            mesh
                        } else {
                            continue;
                        };
                        let position_vertices = mesh.vertex_data::<Position>().unwrap();
                        pass.set_vertex_buffer(0, mesh.buffer.slice(position_vertices.offset..));

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
