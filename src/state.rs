use anyhow::Result;
use cfg_if::cfg_if;
use wgpu::util::DeviceExt;
use winit::{event::*, window::Window};

use crate::enrichments::{gaze::GazeUniform, Enrichments};
use crate::screen::ScreenUniform;

cfg_if!( if #[cfg(feature = "texture")] {
    use crate::texture::Texture;
    use crate::vertex::{Vertex, TEXTURED_PENTAGON}; // @TODO get rid of this
} else {
    use crate::vertex::{Vertex, FULL_SCREEN_QUAD};
});

#[cfg(feature = "camera")]
use crate::camera::{Camera, CameraController, CameraUniform};

#[cfg(feature = "instances")]
use {
    crate::instances::{Instance, InstanceRaw, INSTANCE_DISPLACEMENT, NUM_INSTANCES_PER_ROW},
    cgmath::{InnerSpace, Rotation3, Zero},
};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: winit::window::Window,
    window_physical_size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    #[cfg(not(feature = "texture"))]
    gaze_buffer: wgpu::Buffer,

    #[cfg(not(feature = "texture"))]
    gaze_uniform: GazeUniform,

    #[cfg(not(feature = "texture"))]
    gaze_bind_group: wgpu::BindGroup,

    #[cfg(feature = "texture")]
    _texture: Texture, // unused for now
    #[cfg(feature = "texture")]
    texture_bind_group: wgpu::BindGroup,

    #[cfg(feature = "camera")]
    camera: Camera,
    #[cfg(feature = "camera")]
    camera_buffer: wgpu::Buffer,
    #[cfg(feature = "camera")]
    camera_uniform: CameraUniform,
    #[cfg(feature = "camera")]
    camera_bind_group: wgpu::BindGroup,
    #[cfg(feature = "camera")]
    camera_controller: CameraController,

    #[cfg(feature = "instances")]
    instances: Vec<Instance>,
    #[cfg(feature = "instances")]
    instance_buffer: wgpu::Buffer,

    #[cfg(feature = "depth")]
    depth_texture: Texture,
}

impl State {
    pub async fn new(window: Window, enrichments: Enrichments) -> Self {
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let window_physical_size = window.inner_size();

        // The unsafe API is about to change: https://github.com/gfx-rs/wgpu/issues/1463
        // Winit EventLoop 3.0 Changes: https://github.com/rust-windowing/winit/issues/2900
        // @TODO keep track of the upstream changes and remove this unsafe call
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        // The shader code assumes an sRGB surface texture. Using a different one
        // will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_physical_size.width,
            height: window_physical_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        cfg_if! {
        if #[cfg(feature = "texture")] {
            #[cfg(not(feature = "camera"))]
            let shader_source = include_str!("../assets/shaders/texture.wgsl").into();

            let vertices = TEXTURED_PENTAGON.vertices;
            let indices = TEXTURED_PENTAGON.indices;

            let texture_bytes = include_bytes!("../assets/images/happy-tree.png");
            let texture = Texture::from_bytes(&device, &queue, texture_bytes, "happy_tree").unwrap();

            let texture_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the corresponding BindingType::Texture above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });
            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some("texture_bind_group"),
            });

        } else {
            let screen_uniform = ScreenUniform::new(config.width as f32, config.height as f32);
            let gaze_uniform = GazeUniform::new(enrichments.gaze.unwrap()).unwrap();

            // @TODO they should be two separated bind groups
            let screen_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Screen Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[screen_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }
            );

            let gaze_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Gaze Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[gaze_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }
            );

            let gaze_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("gaze_bind_group_layout"),
            });

            let gaze_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &gaze_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: screen_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: gaze_buffer.as_entire_binding(),
                    }
                ],
                label: Some("gaze_bind_group"),
            });

            let shader_source = include_str!("../assets/shaders/circle.wgsl").into();
            let vertices = FULL_SCREEN_QUAD.vertices;
            let indices = FULL_SCREEN_QUAD.indices;
        }}

        cfg_if! { if #[cfg(feature = "camera")] {
            #[cfg(not(feature = "instances"))]
            let shader_source = include_str!("../assets/shaders/camera.wgsl").into();

            let camera = Camera {
                eye: (0.0, 1.0, 2.0).into(), // position the camera one unit up and 2 back
                target: (0.0, 0.0, 0.0).into(), // look at the origin
                up: cgmath::Vector3::unit_y(), // which way is "up"
                aspect: config.width as f32 / config.height as f32,
                fovy: 45.0,
                znear: 0.1,
                zfar: 100.0,
            };

            let mut camera_uniform = CameraUniform::new();
            camera_uniform.update_view_proj(&camera);

            let camera_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                }
            );

            let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("camera_bind_group_layout"),
            });

            let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ],
                label: Some("camera_bind_group"),
            });

            let camera_speed = 0.05;
            let camera_controller = CameraController::new(camera_speed);

        } else {
        }}

        cfg_if! { if #[cfg(feature = "instances")] {
            let shader_source = include_str!("../assets/shaders/instances.wgsl").into();

            let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - INSTANCE_DISPLACEMENT;

                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can effect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance {
                        position, rotation,
                    }
                })
            }).collect::<Vec<_>>();

            // Make sure if you add new instances to the Vec, that you
            // recreate the instance_buffer and as well as camera_bind_group,
            // otherwise your new instances won't show up correctly.
            let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
            let instance_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );

        } else {
        }}

        cfg_if! { if #[cfg(feature = "depth")] {
            let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
            let depth_stencil = Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            });
        } else {
            let depth_stencil = None;
        }}

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = indices.len() as u32;

        cfg_if! { if #[cfg(feature = "texture")] {
            let bind_group_layouts = &[
                &texture_bind_group_layout, // @group(0)
                #[cfg(feature = "camera")]
                &camera_bind_group_layout, // @group(1)
            ];
        } else {
            let bind_group_layouts = &[
                &gaze_bind_group_layout, // @group(0)
            ];
        }};

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::descriptor(),
                    #[cfg(feature = "instances")]
                    InstanceRaw::descriptor(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                // Triangles with vertices in counter clockwise order are considered the front face.
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            window_physical_size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,

            #[cfg(not(feature = "texture"))]
            gaze_buffer,

            #[cfg(not(feature = "texture"))]
            gaze_uniform,

            #[cfg(not(feature = "texture"))]
            gaze_bind_group,

            #[cfg(feature = "texture")]
            _texture: texture,
            #[cfg(feature = "texture")]
            texture_bind_group,

            #[cfg(feature = "camera")]
            camera,
            #[cfg(feature = "camera")]
            camera_buffer,
            #[cfg(feature = "camera")]
            camera_uniform,
            #[cfg(feature = "camera")]
            camera_bind_group,
            #[cfg(feature = "camera")]
            camera_controller,

            #[cfg(feature = "instances")]
            instances,
            #[cfg(feature = "instances")]
            instance_buffer,

            #[cfg(feature = "depth")]
            depth_texture,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_physical_size = new_size;
            self.config.width = self.window_physical_size.width;
            self.config.height = self.window_physical_size.height;
            self.surface.configure(&self.device, &self.config);

            cfg_if! { if #[cfg(feature = "camera")] {
                self.camera.aspect = self.config.width as f32 / self.config.height as f32;
            } else {
            }}

            cfg_if! { if #[cfg(feature = "depth")] {
                self.depth_texture =
                    Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            } else {
            }}
        }
    }

    pub fn recover(&mut self) {
        self.resize(self.window_physical_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        cfg_if! { if #[cfg(feature = "camera")] {
            self.camera_controller.handle_event(event)
        } else {
            match event {
                _ => false,
            }
        }}
    }

    #[cfg(not(feature = "texture"))]
    pub fn handle_mouse_move_vip_event(&mut self, x: f32, y: f32) {
        self.gaze_uniform.set_position([x, y]);
        self.queue.write_buffer(
            &self.gaze_buffer,
            0,
            bytemuck::cast_slice(&[self.gaze_uniform]),
        );
    }

    pub fn update(&mut self) {
        cfg_if! { if #[cfg(feature = "camera")] {
            self.camera_controller.update_camera(&mut self.camera);
            self.camera_uniform.update_view_proj(&self.camera);
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
        } else {
        }}
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            cfg_if! { if #[cfg(feature = "depth")] {
                let depth_stencil_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                });
            } else {
                let depth_stencil_attachment: Option<wgpu::RenderPassDepthStencilAttachment> = None;
            }}

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment,
            });

            cfg_if! { if #[cfg(feature = "instances")] {
                let instances = 0..self.instances.len() as u32;
            } else {
                let instances = 0..1;
            }}

            render_pass.set_pipeline(&self.render_pipeline);
            cfg_if! { if #[cfg(feature = "texture")] {
                render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
                #[cfg(feature = "camera")]
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            } else {
                render_pass.set_bind_group(0, &self.gaze_bind_group, &[]);
            }}
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            #[cfg(feature = "instances")]
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, instances);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
