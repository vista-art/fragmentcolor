use crate::{
    renderer::{
        target::Dimensions, RenderContext, RenderPass, RenderPassResult, RenderTarget,
        RenderTargetCollection, Renderer,
    },
    scene::SceneState,
};
use std::{mem, sync::RwLockReadGuard};

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
    fn draw(&mut self, scene: RwLockReadGuard<'_, SceneState>) -> RenderPassResult {
        let renderer = self.renderer;
        let targets = renderer.read_targets();
        let device = renderer.device();

        let nodes = scene.get_global_transforms();

        self.uniform_pool.reset();
        let queue = renderer.queue();

        let mut commands = Vec::new();

        let mut rendered_frames = Vec::new();
        for (object_id, camera) in scene.cameras().iter() {
            let camera_targets = scene.get_camera_targets(object_id);

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
                    //if camera_target.clip_region.is_smaller_than(target.size()) {

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
                        "Clip Region not implemented: Camera {:?} is targeting a target {:?} with a different size.",
                        object_id,
                        camera_target.target_id
                    );
                }

                let cam_node = &nodes[camera.node_id];

                {
                    let projection_m = camera.projection_matrix(target.aspect());
                    let inverse_m = cam_node.inverse_matrix();
                    let final_m = glam::Mat4::from(projection_m) * glam::Mat4::from(inverse_m);
                    let globals = Globals {
                        view_proj: final_m.to_cols_array_2d(),
                    };
                    queue.write_buffer(&self.global_uniform_buf, 0, bytemuck::bytes_of(&globals));
                }

                self.temp.clear();
                self.uniform_pool.reset();
                let cam_dir = glam::Quat::from_slice(&cam_node.rotation) * -glam::Vec3::Z;

                // gather all sprites
                for (_, sprite) in scene.sprites().iter() {
                    if let Some(sprite_image) = sprite.image {
                        let local = &nodes[sprite.node_id];
                        let cam_vector = glam::Vec3::from_slice(&local.position)
                            - glam::Vec3::from_slice(&cam_node.position);
                        let camera_distance = cam_vector.dot(cam_dir);

                        let resources = renderer.read_resources();
                        let image = if let Some(image) = resources.get_texture(&sprite_image) {
                            image
                        } else {
                            log::error!(
                                "Sprite {:?} is using a non-existent texture {:?}",
                                sprite.node_id,
                                sprite_image
                            );
                            // @TODO use or generate a default texture
                            continue;
                        };
                        let locals = Locals {
                            position: local.position,
                            scale: local.scale,
                            rotation: local.rotation,
                            bounds: {
                                let (width, heigth) = match sprite.clip_region {
                                    Some(ref clip) => (clip.width(), clip.height()),
                                    None => (image.size.width, image.size.height),
                                };
                                [
                                    -0.5 * width as f32,
                                    -0.5 * heigth as f32,
                                    0.5 * width as f32,
                                    0.5 * heigth as f32,
                                ]
                            },
                            tex_coords: match sprite.clip_region {
                                Some(ref clip) => [
                                    clip.x_min as f32 / image.size.width as f32,
                                    clip.y_min as f32 / image.size.height as f32,
                                    clip.x_max as f32 / image.size.width as f32,
                                    clip.y_max as f32 / image.size.height as f32,
                                ],
                                None => [0.0, 0.0, 1.0, 1.0],
                            },
                        };
                        let locals_bl = self.uniform_pool.alloc(&locals, queue);

                        // pre-create local bind group, if needed
                        let local_bgl = &self.local_bind_group_layout;
                        let key = LocalKey {
                            uniform_buf_index: locals_bl.index,
                            image: sprite_image,
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
                            image: sprite_image,
                        });
                    }
                }

                // sort from back to front
                self.temp
                    .sort_by_key(|s| (s.camera_distance * -1000.0) as i64);

                // @TODO if this target is targeted by multiple cameras,
                //       we have to save the results and start a new renderpass
                //       with Ops: Store
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
                                load: wgpu::LoadOp::Clear(camera_target.clear_color.into()),
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
                target.prepare_render(renderer, &mut commands);

                rendered_frames.push((target.id(), frame));
            }
        }

        Ok((commands, rendered_frames))
    }
}
