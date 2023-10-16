use crate::renderer::{
    resources::{
        mesh::{Mesh, MeshBuilder, MeshRef},
        sampler::create_default_sampler,
    },
    target::{FrameTarget, HasWindow, Target, TargetInfo, TargetRef, WindowTarget},
    texture::{Texture, TextureRef},
    RenderPass,
};
use crate::scene::{camera::Camera, Scene};
use std::{fs::File, io, path::Path};
use wgpu::util::DeviceExt;

type Error = Box<dyn std::error::Error>;

/// Trait that exposes `Context` details that depend on `wgpu`
pub trait RenderContext {
    fn get_texture(&self, ir: TextureRef) -> &Texture;
    fn get_target(&self, tr: TargetRef) -> &Target;
    fn get_mesh(&self, mr: MeshRef) -> &Mesh;
    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
}

#[derive(Default, Debug)]
pub struct RendererBuilder {
    power_preference: wgpu::PowerPreference,
    software: bool,
}

pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) targets: Vec<Target>,
    pub(crate) window: Option<WindowTarget>, // @TODO: remove this (will be part of Targets)
    pub(crate) resources: Resources,
}

// NOTE: If you ever need to refine this, look
// at Ruffle's TexturePool and BufferPool structs
pub struct Resources {
    pub(crate) textures: Vec<Texture>,
    pub(crate) meshes: Vec<Mesh>,
}

impl RenderContext for Renderer {
    fn get_texture(&self, ir: TextureRef) -> &Texture {
        &self.resources.textures[ir.0 as usize]
    }
    fn get_target(&self, tr: TargetRef) -> &Target {
        &self.targets[tr.0 as usize]
    }
    fn get_mesh(&self, mr: MeshRef) -> &Mesh {
        &self.resources.meshes[mr.0 as usize]
    }
    fn device(&self) -> &wgpu::Device {
        &self.device
    }
    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

impl Renderer {
    // @TODO: remove this
    pub fn init() -> RendererBuilder {
        RendererBuilder::default()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // @TODO Here we should loop all the targets and resize them
        let window = match self.window {
            Some(ref mut suf) => suf,
            None => return,
        };
        if (window.config.width, window.config.height) == (width, height) {
            return;
        }
        window.config.width = width;
        window.config.height = height;
        window.surface.configure(&self.device, &window.config);

        // for target in self.targets.iter_mut() {
        //     target.resize(&self.device, width, height);
        // }
    }

    pub fn present<P: RenderPass>(
        &mut self,
        pass: &mut P,
        scene: &Scene,
        camera: &Camera,
    ) -> Result<(), Error> {
        let window = self.window.as_mut().expect("No screen is configured!");

        // @TODO implement Ruffle's interface for render targets
        let frame = window.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let tr = TargetRef(self.targets.len() as _);
        self.targets.push(Target::Frame(FrameTarget {
            view,
            format: frame.texture.format(),
            size: wgpu::Extent3d {
                width: window.config.width,
                height: window.config.height,
                depth_or_array_layers: 1,
            },
        }));

        // @TODO multiple passes
        pass.draw(&[tr], scene, camera, self);

        self.targets.pop();
        frame.present();

        Ok(())
    }

    // @TODO remove this
    pub fn add_mesh(&mut self) -> MeshBuilder {
        MeshBuilder::new(self)
    }

    // @TODO remove this
    pub fn surface_info(&self) -> Option<TargetInfo> {
        self.window.as_ref().map(|s| TargetInfo {
            format: s.config.format,
            sample_count: 1,
            aspect_ratio: s.config.width as f32 / s.config.height as f32,
        })
    }

    pub fn load_image(&mut self, path_ref: impl AsRef<Path>) -> TextureRef {
        let path = path_ref.as_ref();
        let image_format = image::ImageFormat::from_extension(path.extension().unwrap())
            .unwrap_or_else(|| panic!("Unrecognized image extension: {:?}", path.extension()));

        let label = path.display().to_string();
        let file = File::open(path)
            .unwrap_or_else(|e| panic!("Unable to open {}: {:?}", path.display(), e));
        let mut buf_reader = io::BufReader::new(file);

        let (texture, size) = if image_format == image::ImageFormat::Dds {
            let dds = ddsfile::Dds::read(&mut buf_reader)
                .unwrap_or_else(|e| panic!("Unable to read {}: {:?}", path.display(), e));

            println!("Header {:?}", dds.header);
            let mip_level_count = dds.get_num_mipmap_levels();
            let (dimension, depth_or_array_layers) = match dds.header10 {
                Some(ref h) => match h.resource_dimension {
                    ddsfile::D3D10ResourceDimension::Texture2D => {
                        (wgpu::TextureDimension::D2, h.array_size)
                    }
                    ddsfile::D3D10ResourceDimension::Texture3D => {
                        (wgpu::TextureDimension::D3, dds.get_depth())
                    }
                    other => panic!("Unsupported resource dimension {:?}", other),
                },
                None => match dds.header.depth {
                    None | Some(1) => (wgpu::TextureDimension::D2, 1),
                    Some(other) => (wgpu::TextureDimension::D3, other),
                },
            };

            let format = if let Some(fourcc) = dds.header.spf.fourcc {
                match fourcc.0 {
                    ddsfile::FourCC::BC1_UNORM => wgpu::TextureFormat::Bc1RgbaUnormSrgb,
                    ddsfile::FourCC::BC2_UNORM => wgpu::TextureFormat::Bc2RgbaUnormSrgb,
                    ddsfile::FourCC::BC3_UNORM => wgpu::TextureFormat::Bc3RgbaUnormSrgb,
                    ddsfile::FourCC::BC4_UNORM => wgpu::TextureFormat::Bc4RUnorm,
                    ddsfile::FourCC::BC4_SNORM => wgpu::TextureFormat::Bc4RSnorm,
                    ddsfile::FourCC::BC5_UNORM => wgpu::TextureFormat::Bc5RgUnorm,
                    ddsfile::FourCC::BC5_SNORM => wgpu::TextureFormat::Bc5RgSnorm,
                    ref other => panic!("Unsupported DDS FourCC {:?}", other),
                }
            } else {
                assert_eq!(dds.header.spf.rgb_bit_count, Some(32));
                wgpu::TextureFormat::Rgba8UnormSrgb
            };

            let desc = wgpu::TextureDescriptor {
                label: Some(&label),
                size: wgpu::Extent3d {
                    width: dds.header.width,
                    height: dds.header.height,
                    depth_or_array_layers,
                },
                mip_level_count,
                sample_count: 1,
                dimension,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[format],
            };
            let texture = self
                .device
                .create_texture_with_data(&self.queue, &desc, &dds.data);

            (texture, desc.size)
        } else {
            let img = image::load(buf_reader, image_format)
                .unwrap_or_else(|e| panic!("Unable to decode {}: {:?}", path.display(), e))
                .to_rgba8();

            let (width, height) = img.dimensions();
            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };
            let desc = wgpu::TextureDescriptor {
                label: Some(&label),
                size,
                mip_level_count: 1, //TODO: generate `size.max_mips()` mipmaps
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
            };
            let texture = self.device.create_texture(&desc);

            self.queue.write_texture(
                texture.as_image_copy(),
                &img,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: None,
                },
                size,
            );
            (texture, size)
        };

        self.add_texture(texture, size)
    }

    // @TODO receive our texture instead
    pub fn add_texture(&mut self, texture: wgpu::Texture, size: wgpu::Extent3d) -> TextureRef {
        let index = self.resources.textures.len();
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = create_default_sampler(&self.device);
        let format = texture.format();
        self.resources.textures.push(Texture {
            data: texture,
            view,
            size,
            format,
            sampler,
        });
        TextureRef(index as u32)
    }

    pub fn add_image_from_bytes(
        &mut self,
        desc: &wgpu::TextureDescriptor,
        data: &[u8],
    ) -> TextureRef {
        let texture = self
            .device
            .create_texture_with_data(&self.queue, &desc, data);

        self.add_texture(texture, desc.size.clone())
    }
}

impl RendererBuilder {
    pub fn power_hungry(self, hungry: bool) -> Self {
        Self {
            power_preference: if hungry {
                wgpu::PowerPreference::HighPerformance
            } else {
                wgpu::PowerPreference::LowPower
            },
            ..self
        }
    }

    pub fn software(self, software: bool) -> Self {
        Self { software, ..self }
    }

    pub async fn build_offscreen(self) -> Renderer {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: self.power_preference,
                force_fallback_adapter: self.software,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let resources = Resources {
            textures: Vec::new(),
            meshes: Vec::new(),
        };

        Renderer {
            device,
            queue,
            targets: Vec::new(),
            resources,
            window: None,
        }
    }

    pub async fn build<W: HasWindow>(self, window: &W) -> Renderer {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let size = window.size();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: self.power_preference,
                force_fallback_adapter: self.software,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            alpha_mode,
            present_mode: surface_capabilities.present_modes[0],
            view_formats: vec![surface_format],
        };

        surface.configure(&device, &config);

        let window = WindowTarget { surface, config };

        // @TODO add window to targets list
        //       Targets could be an enum.
        // let targets = vec![Target {
        //     size,
        //     view: window
        //         .surface
        //         .get_current_texture()?
        //         .texture
        //         .create_view(&wgpu::TextureViewDescriptor::default()),
        //     format: config.format,
        //     window: Some(window),
        // }];

        let resources = Resources {
            textures: Vec::new(),
            meshes: Vec::new(),
        };

        Renderer {
            device,
            queue,
            window: Some(window),
            targets: Vec::new(),
            resources,
        }
    }
}
