use crate::{
    renderer::{
        renderpass::buffer, target::Dimensions, IsRenderTarget, RenderContext, RenderPass,
        RenderPassResult, RenderTargetCollection, Renderer,
    },
    scene::SceneState,
    IsHidden, Sprite,
};
use std::{mem, sync::RwLockReadGuard};

/// @Group(0) @Binding(0)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct WindowUniforms {
    resolution: [f32; 2],
    antialiaser: f32,
    fps: f32,
    time: f32,
    frame_delta: f32,
    mouse: [f32; 2],
    drag_start: [f32; 2],
    drag_end: [f32; 2],
    mouse_left_pressed: f32,
    mouse_left_clicked: f32,
}

/// @Group(0) @Binding(1)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
}

/// @Group(0) @Binding(2)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Locals {
    position: [f32; 4],
    rotation: [f32; 4],
    scale: [f32; 4],
    color: [f32; 4],
    bounds: [f32; 4],
    radius: f32, // derived from bounds - border
    border: f32,
    padding: f32, // unused; needed for alignment
    sdf_flags: f32,
    texture_uv: [f32; 4],
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
    locals_bl: buffer::BufferLocation,
    image: crate::TextureId,
}

pub(crate) struct Toy<'r> {
    renderer: &'r Renderer,
    window_uniform_buffer: wgpu::Buffer,
    globals_uniform_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
    locals_bind_group_layout: wgpu::BindGroupLayout,
    locals_bind_groups: fxhash::FxHashMap<LocalKey, wgpu::BindGroup>,
    uniform_pool: buffer::BufferPool,
    pipelines: Pipelines,
    temp: Vec<Instance>,
}

impl<'r> Toy<'r> {
    pub(crate) fn new(renderer: &'r Renderer) -> Self {
        let device = renderer.device();

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Toy Renderpass: Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("toy.wgsl").into()),
        });

        // @TODO When we implement shader composition, this should be
        //       shared between all RenderPasses so we don't need to
        //       crate a custom one for each RenderPass. They can still
        //       have their own custom bindings, maybe in Group(1).
        let window_buffer_size = mem::size_of::<WindowUniforms>() as wgpu::BufferAddress;
        let globals_buffer_size = mem::size_of::<Globals>() as wgpu::BufferAddress;

        let global_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Toy Renderpass Bind Group Layout: @Group(0): Window(0) & Globals(1)"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(window_buffer_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(globals_buffer_size),
                    },
                    count: None,
                },
            ],
        });

        let window_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Toy Renderpass: Window Uniform Buffer"),
            size: window_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Toy Renderpass: Globals Uniform Buffer"),
            size: globals_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Toy Renderpass: Global Bind Group"),
            layout: &global_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: window_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: globals_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let locals_size = mem::size_of::<Locals>() as wgpu::BufferAddress;
        let local_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Toy Renderpass: Locals Bind Group Layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Toy Renderpass: Pipeline Layout"),
            bind_group_layouts: &[&global_bgl, &local_bgl],
            push_constant_ranges: &[],
        });

        let pipelines = {
            let mut sample_count = 1;
            let targets = &renderer
                .read_targets()
                // @TODO I MEAN IT!!! Remove tech debt (global search "TECH DEBT")
                .expect(
                    "TECH DEBT: Avoid panics 

                (after you see smtg in the screen, NOT NOW)!!!
                
                ",
                )
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
            window_uniform_buffer,
            globals_uniform_buffer,
            globals_bind_group,
            locals_bind_groups: Default::default(),
            locals_bind_group_layout: local_bgl,
            uniform_pool: buffer::BufferPool::uniform("Toy VertexInput Buffer Pool", &device),
            pipelines,
            temp: Vec::new(),
        }
    }
}

impl<'r> RenderPass for Toy<'r> {
    fn draw(&mut self, scene: RwLockReadGuard<'_, SceneState>) -> RenderPassResult {
        self.uniform_pool.reset();
        let renderer = self.renderer;
        let targets = renderer
            .read_targets()
            // @TODO I MEAN IT!!! Remove tech debt (global search "TECH DEBT")
            .expect("TECH DEBT: Avoid panics!!!");
        let device = renderer.device();
        let queue = renderer.queue();

        let transforms = scene.calculate_global_transforms();

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

                let resolution = target.size();
                let antialiaser = resolution.antialias_factor();
                let cam_transform = &transforms[camera.transform_id];
                {
                    let projection_m = camera.projection_matrix(target.aspect());
                    let inverse_m = cam_transform.inverse_matrix();
                    let final_m = glam::Mat4::from(projection_m) * glam::Mat4::from(inverse_m);

                    // @TODO fill this with real values
                    let window_uniforms = WindowUniforms {
                        resolution: [resolution.width() as f32, resolution.height() as f32],
                        antialiaser,
                        fps: 0.0,                // @TODO (fps is unimplemented;
                        time: 0.0,               // @TODO (playback time is unimplemented;
                        frame_delta: 0.0,        // @TODO (playback time is unimplemented;
                        mouse: [0.0, 0.0],       // @TODO (mouse is unimplemented;
                        drag_start: [0.0, 0.0],  // @TODO (mouse is unimplemented;
                        drag_end: [0.0, 0.0],    // @TODO (mouse is unimplemented;
                        mouse_left_pressed: 0.0, // @TODO (mouse is unimplemented;
                        mouse_left_clicked: 0.0, // @TODO (mouse is unimplemented;
                    };
                    queue.write_buffer(
                        &self.window_uniform_buffer,
                        0,
                        bytemuck::bytes_of(&window_uniforms),
                    );

                    let globals = Globals {
                        view_proj: final_m.to_cols_array_2d(),
                    };
                    queue.write_buffer(
                        &self.globals_uniform_buffer,
                        0,
                        bytemuck::bytes_of(&globals),
                    );
                }

                self.temp.clear();
                self.uniform_pool.reset();
                let cam_dir = glam::Quat::from_slice(&cam_transform.rotation) * -glam::Vec3::Z;

                // Gather all 2D Shapes...
                for (object_id, (color, bounds, border, shape_flag)) in
                    scene.get_2d_objects().without::<&IsHidden>().iter()
                {
                    // Sprites must be rendered first
                    if let Ok(sprite) = scene.world.get::<&Sprite>(object_id) {
                        let local_transform = &transforms[sprite.transform_id];

                        let camera_vector = glam::Vec3::from_slice(&local_transform.position)
                            - glam::Vec3::from_slice(&cam_transform.position);
                        let camera_distance = camera_vector.dot(cam_dir);

                        let resources = if let Ok(resources) = renderer.read_resources() {
                            resources
                        } else {
                            log::error!(
                                "Failed to read resources for Sprite {:?}. Skipping Sprite...",
                                sprite
                            );
                            continue;
                        };

                        let image = if let Some(image) = resources.get_texture(&sprite.image) {
                            image
                        } else {
                            log::error!(
                                "Sprite {:?} is using a non-existent texture {:?}",
                                sprite,
                                sprite.image
                            );
                            continue;
                        };

                        let locals = Locals {
                            position: local_transform.position,
                            rotation: local_transform.rotation,
                            scale: local_transform.scale,
                            color: color.to_array(),
                            bounds: bounds.0.to_array(),
                            texture_uv: sprite.clip_region.unwrap_or(bounds.0).to_array(),
                            radius: bounds.0.inbound_radius() - border.0,
                            border: border.0,
                            padding: 0.0, // unused; needed for alignment
                            sdf_flags: shape_flag.0,
                        };
                        let locals_bl = self.uniform_pool.alloc(&locals, queue);
                        let local_bgl = &self.locals_bind_group_layout;

                        // pre-create local bind group, if needed
                        let key = LocalKey {
                            uniform_buf_index: locals_bl.index,
                            image: sprite.image,
                        };

                        let binding = self.uniform_pool.binding::<Locals>(locals_bl.index);
                        self.locals_bind_groups.entry(key).or_insert_with(|| {
                            device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Toy VertexInput Bind Group Descriptor"),
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
                                    wgpu::BindGroupEntry {
                                        binding: 2,
                                        resource: wgpu::BindingResource::Sampler(&image.sampler),
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

                // @TODO this is the core of what the RenderPass does, and it only needs a Frame
                //       from a specific target. The RenderPass trait abstraction for multiple targets
                //       is wrong, I should go back to the older method or craate a second trait for a single target.
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Toy Render Pass"),
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
                    pass.set_bind_group(0, &self.globals_bind_group, &[]);

                    for inst in self.temp.drain(..) {
                        let key = LocalKey {
                            uniform_buf_index: inst.locals_bl.index,
                            image: inst.image,
                        };
                        let local_bg = &self.locals_bind_groups[&key];
                        pass.set_bind_group(1, local_bg, &[inst.locals_bl.offset]);
                        pass.draw(0..3, 0..1); // @TODO this should be indexed, shapes are not sprites
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
