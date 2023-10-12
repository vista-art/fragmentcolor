use crate::renderer::{RenderableRefs, RenderableTrait};
use cfg_if::cfg_if;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::renderer::debug::texture::Texture;
use crate::renderer::screen::ScreenUniform;
use crate::scene::model::{ShapeVertex, Vertex};
use std::collections::HashMap;

use crate::scene::camera::{Camera, CameraController, CameraUniform};
use {
    crate::scene::instances::{
        Instance, InstanceRaw, INSTANCE_DISPLACEMENT, NUM_INSTANCES_PER_ROW,
    },
    cgmath::{InnerSpace, Rotation3, Zero},
};

pub struct GPU {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct Target {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

impl GPU {
    pub fn new() -> Self {
        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        Self { device, queue }
    }
}

#[derive(Debug)]
pub(super) struct Pipeline {
    // scene
    pub(super) vertex_buffer: wgpu::Buffer,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) num_indices: u32,

    // scene
    pub(super) instances: Vec<Instance>,
    pub(super) instance_buffer: wgpu::Buffer,

    // includes camera buffer and camera bind group
    pub(super) uniform_buffers: HashMap<String, wgpu::Buffer>,
    pub(super) bind_groups: HashMap<String, wgpu::BindGroup>,

    // scene
    // consider making this a collection
    pub(super) render_pipeline: wgpu::RenderPipeline,

    pub(super) textures: Vec<Texture>,
    pub(super) texture_bind_group: wgpu::BindGroup,

    pub(super) camera: Camera,
    pub(super) camera_uniform: CameraUniform,
    pub(super) camera_bind_group: wgpu::BindGroup,
    pub(super) camera_controller: CameraController,
    pub(super) depth_texture: Texture,
}

impl State {
    pub async fn new(window: &Window, renderables: &RenderableRefs) -> State {
        // The unsafe API is about to change: https://github.com/gfx-rs/wgpu/issues/1463
        // Winit EventLoop 3.0 Changes: https://github.com/rust-windowing/winit/issues/2900
        // @TODO keep track of the upstream changes and remove this unsafe call
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find a GPU adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(wasm) {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .expect("Failed to create device");

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

        // alpha_mode should be transparent if the surface supports it
        let alpha_mode = surface_capabilities
            .alpha_modes
            .iter()
            .find(|m| *m == &wgpu::CompositeAlphaMode::PreMultiplied)
            .unwrap_or(&wgpu::CompositeAlphaMode::Auto)
            .to_owned();

        let window_physical_size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            width: window_physical_size.width,
            height: window_physical_size.height,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode,
            format: surface_format,
            view_formats: &[],
        };

        surface.configure(&device, &config);

        // Globals
        let screen_uniform = ScreenUniform::new(config.width as f32, config.height as f32);
        let screen_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen Uniform Buffer"),
            contents: bytemuck::cast_slice(&[screen_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let screen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Screen Bind Group Layout"),
            });
        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buffer.as_entire_binding(),
            }],
            label: Some("Screen Bind Group"),
        });

        let mut buffers: HashMap<String, wgpu::Buffer> = HashMap::new();
        let mut bind_groups: HashMap<String, wgpu::BindGroup> = HashMap::new();
        let mut bind_group_layouts: Vec<&wgpu::BindGroupLayout> = vec![];

        // Items here are referenced by the GPU shader in
        // WGSL using the `@group(n)` syntax, where `n` is
        // the index of the item in this list.
        bind_group_layouts.push(&screen_bind_group_layout);
        buffers.insert("Screen".to_string(), screen_buffer);
        bind_groups.insert("Screen".to_string(), screen_bind_group);

        let mut renderables_bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry> = vec![];
        let mut renderables_bind_group_entries: Vec<wgpu::BindGroupEntry> = vec![];

        for renderable in renderables.iter() {
            let label = renderable.read().unwrap().label();
            let buffer = renderable.read().unwrap().buffer(&device);
            buffers.insert(label.to_string(), buffer);
        }

        for (i, renderable) in renderables.iter().enumerate() {
            let label = renderable.read().unwrap().label();

            let renderables_bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
            let renderables_bind_group_entry = wgpu::BindGroupEntry {
                binding: i as u32,
                resource: buffers[&label].as_entire_binding(),
            };

            renderables_bind_group_layout_entries.push(renderables_bind_group_layout_entry);
            renderables_bind_group_entries.push(renderables_bind_group_entry);
        }

        let renderables_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: renderables_bind_group_layout_entries.as_slice(),
                label: Some("Renderables Bind Group Layout"),
            });

        let renderables_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderables_bind_group_layout,
            entries: renderables_bind_group_entries.as_slice(),
            label: Some("Renderables Bind Group"),
        });

        bind_group_layouts.push(&renderables_bind_group_layout);
        let bind_group_layouts = bind_group_layouts.as_slice();

        bind_groups.insert("Renderables".to_string(), renderables_bind_group);

        // @TODO every renderable should reference their own shaders, vertices, indices and textures
        let shader_source = include_str!("./shaders/circle.wgsl").into();
        let vertices = FULL_SCREEN_QUAD.vertices;
        let indices = FULL_SCREEN_QUAD.indices;

        // @TODO load the texture from the root dir (needs build.rs)
        //       check: https://sotrh.github.io/learn-wgpu/beginner/tutorial9-models/#accessing-files-in-the-res-folder
        let texture_bytes = include_bytes!("./images/happy-tree.png");
        let textures =
            vec![Texture::from_bytes(&device, &queue, texture_bytes, "happy_tree").unwrap()];
        let texture = &textures[0];

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

        // this will come from the scene
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
        camera_uniform.update(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let camera_speed = 0.05;
        let camera_controller = CameraController::new(camera_speed);

        // @TODO this will come from the model
        let has_instances = true;
        if has_instances {
            let instances = (0..NUM_INSTANCES_PER_ROW)
                .flat_map(|z| {
                    (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                        let position = cgmath::Vector3 {
                            x: x as f32,
                            y: 0.0,
                            z: z as f32,
                        } - INSTANCE_DISPLACEMENT;

                        let rotation = if position.is_zero() {
                            // this is needed so an object at (0, 0, 0) won't get scaled to zero
                            // as Quaternions can effect scale if they're not created correctly
                            cgmath::Quaternion::from_axis_angle(
                                cgmath::Vector3::unit_z(),
                                cgmath::Deg(0.0),
                            )
                        } else {
                            cgmath::Quaternion::from_axis_angle(
                                position.normalize(),
                                cgmath::Deg(45.0),
                            )
                        };

                        Instance { position, rotation }
                    })
                })
                .collect::<Vec<_>>();

            // Make sure if you add new instances to the Vec, that you
            // recreate the instance_buffer and as well as camera_bind_group,
            // otherwise your new instances won't show up correctly.
            let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
            let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
        }

        // @TODO this will come from the scene
        let has_depth = true;
        if has_depth {
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
        }

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

        #[cfg(feature = "texture")]
        let bind_group_layouts = &[
            &texture_bind_group_layout, // @group(0)
            &camera_bind_group_layout,  // @group(1)
        ];

        // convert bind_group_layouts to a slice
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
                buffers: &[ModelVertex::descriptor(), InstanceRaw::descriptor()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    // enable alpha blending
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            device,
            queue,

            surface,
            config,

            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,

            buffers,
            bind_groups,

            _textures: textures,
            texture_bind_group,

            camera,
            camera_buffer,
            camera_uniform,
            camera_bind_group,
            camera_controller,

            instances,
            instance_buffer,

            depth_texture,
        }
    }
}
