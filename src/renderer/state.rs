use crate::renderer::renderable::Renderables;
use cfg_if::cfg_if;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use winit::window::Window;

cfg_if!( if #[cfg(feature = "texture")] {
    use crate::renderer::texture::Texture;
    use crate::renderer::vertex::{Vertex, TEXTURED_PENTAGON};
} else {
    use crate::renderer::screen::ScreenUniform; //@TODO make it common or define a globals shader
    use crate::renderer::vertex::{Vertex, FULL_SCREEN_QUAD};
});

#[cfg(feature = "camera")]
use crate::renderer::camera::{Camera, CameraController, CameraUniform};

#[cfg(feature = "instances")]
use {
    crate::renderer::instances::{
        Instance, InstanceRaw, INSTANCE_DISPLACEMENT, NUM_INSTANCES_PER_ROW,
    },
    cgmath::{InnerSpace, Rotation3, Zero},
};

#[derive(Debug)]
pub(super) struct State {
    pub(super) surface: wgpu::Surface,
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) config: wgpu::SurfaceConfiguration,
    pub(super) render_pipeline: wgpu::RenderPipeline,
    pub(super) vertex_buffer: wgpu::Buffer,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) num_indices: u32,

    pub(super) buffers: HashMap<String, wgpu::Buffer>,
    pub(super) bind_groups: HashMap<String, wgpu::BindGroup>,

    // manually-enabled features
    // for quick visual testing
    #[cfg(feature = "texture")]
    pub(super) _textures: Vec<Texture>,
    #[cfg(feature = "texture")]
    pub(super) texture_bind_group: wgpu::BindGroup,

    #[cfg(feature = "camera")]
    pub(super) camera: Camera,
    #[cfg(feature = "camera")]
    pub(super) camera_buffer: wgpu::Buffer,
    #[cfg(feature = "camera")]
    pub(super) camera_uniform: CameraUniform,
    #[cfg(feature = "camera")]
    pub(super) camera_bind_group: wgpu::BindGroup,
    #[cfg(feature = "camera")]
    pub(super) camera_controller: CameraController,

    #[cfg(feature = "instances")]
    pub(super) instances: Vec<Instance>,
    #[cfg(feature = "instances")]
    pub(super) instance_buffer: wgpu::Buffer,

    #[cfg(feature = "depth")]
    pub(super) depth_texture: Texture,
}

impl State {
    pub async fn new(window: &Window, renderables: &Renderables) -> State {
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

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

        let window_physical_size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            width: window_physical_size.width,
            height: window_physical_size.height,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            format: surface_format,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        //cfg_if! { if #[cfg(not(feature = "texture"))] {

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

        //let uniforms: Vec<Uniform> = vec![];
        let mut buffers: HashMap<String, wgpu::Buffer> = HashMap::new();
        let mut bind_groups: HashMap<String, wgpu::BindGroup> = HashMap::new();
        let mut bind_group_layouts: Vec<wgpu::BindGroupLayout> = vec![];

        // Items here are referenced by the GPU shader in
        // WGSL using the `@group(n)` syntax, where `n` is
        // the index of the item in this list.
        bind_group_layouts.push(screen_bind_group_layout);
        bind_groups.insert("Screen".to_string(), screen_bind_group);

        for (i, renderable) in renderables.iter().enumerate() {
            //let buffer = renderable.buffer();
            //let uniform = renderable.uniform();
            //let bind_group = renderable.bind_group();
            //let bind_group_layout = renderable.bind_group_layout();

            // buffers.push(buffer);
            // uniforms.push(uniform);
            // bind_groups.push(bind_group);

            let label = renderable.label();
            let renderable_buffer = renderable.buffer(&device);

            // let renderable_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            //     label: Some(format!("{} Uniform Buffer", &label).as_str()),
            //     contents: bytemuck::cast_slice(&[*renderable_uniform]),
            //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            // });

            let renderable_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: i as u32,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some(format!("{} Bind Group Layout", &label).as_str()),
                });

            let renderable_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &renderable_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: renderable_buffer.as_entire_binding(),
                }],
                label: Some(format!("{} Bind Group", &label).as_str()),
            });

            bind_group_layouts.push(renderable_bind_group_layout);
            bind_groups.insert(label.to_string(), renderable_bind_group);
            buffers.insert(label.to_string(), renderable_buffer);
        }

        let bind_group_layouts_refs = bind_group_layouts.iter().collect::<Vec<_>>();

        // @TODO every enrichment should reference their own shader.
        //       Better yet, we should have a "shapes" module that
        //       defines all the basic shapes we'll use, and the
        //       enrichments would use and combine them.
        let shader_source = include_str!("../../assets/shaders/circle.wgsl").into();
        let vertices = FULL_SCREEN_QUAD.vertices;
        let indices = FULL_SCREEN_QUAD.indices;

        cfg_if! { if #[cfg(not(feature = "texture"))] {
        } else {
            #[cfg(not(feature = "camera"))]
            let shader_source = include_str!("../assets/shaders/texture.wgsl").into();

            let vertices = TEXTURED_PENTAGON.vertices;
            let indices = TEXTURED_PENTAGON.indices;

            let texture_bytes = include_bytes!("../assets/images/happy-tree.png");
            let textures = vec![
                Texture::from_bytes(&device, &queue, texture_bytes, "happy_tree").unwrap(),
            ];
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
            camera_uniform.update(&camera);

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

        #[cfg(feature = "texture")]
        let bind_group_layouts = &[
            &texture_bind_group_layout, // @group(0)
            #[cfg(feature = "camera")]
            &camera_bind_group_layout, // @group(1)
        ];

        #[cfg(not(feature = "texture"))]
        // convert bind_group_layouts to a slice
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: bind_group_layouts_refs.as_slice(),
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
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,

            buffers,
            bind_groups,

            #[cfg(feature = "texture")]
            _textures: textures,
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
}
