use crate::gpu::{
    renderpass::Pass,
    target::{HasWindow, SurfaceContext, Target, TargetInfo, TargetRef},
    texture::{Image, ImageInfo, ImageRef},
};
use crate::scene::{
    camera::Camera,
    mesh::{Mesh, MeshBuilder, MeshRef},
    Scene,
};
use std::{fs::File, io, path::Path};
use wgpu::util::DeviceExt;

/// Trait that exposes `Context` details that depend on `wgpu`
pub trait ContextDetail {
    fn get_target(&self, tr: TargetRef) -> &Target;
    fn get_mesh(&self, mr: MeshRef) -> &Mesh;
    fn get_image(&self, ir: ImageRef) -> &Image;
    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
}

#[derive(Default, Debug)]
pub struct ContextBuilder {
    power_preference: wgpu::PowerPreference,
    software: bool,
}

pub struct Context {
    #[allow(unused)]
    pub(super) instance: wgpu::Instance,
    pub(crate) surface: Option<SurfaceContext>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) targets: Vec<Target>,
    pub(crate) images: Vec<Image>,
    pub(crate) meshes: Vec<Mesh>,
}

impl ContextDetail for Context {
    fn get_target(&self, tr: TargetRef) -> &Target {
        &self.targets[tr.0 as usize]
    }
    fn get_mesh(&self, mr: MeshRef) -> &Mesh {
        &self.meshes[mr.0 as usize]
    }
    fn get_image(&self, ir: ImageRef) -> &Image {
        &self.images[ir.0 as usize]
    }
    fn device(&self) -> &wgpu::Device {
        &self.device
    }
    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

impl Context {
    pub fn init() -> ContextBuilder {
        ContextBuilder::default()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let surface = match self.surface {
            Some(ref mut suf) => suf,
            None => return,
        };
        if (surface.config.width, surface.config.height) == (width, height) {
            return;
        }
        surface.config.width = width;
        surface.config.height = height;
        surface.instance.configure(&self.device, &surface.config);
    }

    pub fn present<P: Pass>(&mut self, pass: &mut P, scene: &Scene, camera: &Camera) {
        let surface = self.surface.as_mut().expect("No screen is configured!");
        let frame = surface.instance.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let tr = TargetRef(self.targets.len() as _);
        self.targets.push(Target {
            view,
            format: surface.config.format,
            size: wgpu::Extent3d {
                width: surface.config.width,
                height: surface.config.height,
                depth_or_array_layers: 1,
            },
        });

        pass.draw(&[tr], scene, camera, self);

        self.targets.pop();
        frame.present();
    }

    pub fn add_mesh(&mut self) -> MeshBuilder {
        MeshBuilder::new(self)
    }

    pub fn surface_info(&self) -> Option<TargetInfo> {
        self.surface.as_ref().map(|s| TargetInfo {
            format: s.config.format,
            sample_count: 1,
            aspect_ratio: s.config.width as f32 / s.config.height as f32,
        })
    }

    pub fn get_image_info(&self, image_ref: ImageRef) -> ImageInfo {
        let image = &self.images[image_ref.0 as usize];
        ImageInfo {
            size: [image.size.width as i16, image.size.height as i16].into(),
        }
    }

    pub fn load_image(&mut self, path_ref: impl AsRef<Path>) -> ImageRef {
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

        self.add_image_from_texture(texture, size)
    }

    pub fn add_image_from_texture(
        &mut self,
        texture: wgpu::Texture,
        size: wgpu::Extent3d,
    ) -> ImageRef {
        let index = self.images.len();
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.images.push(Image { view, size });
        ImageRef(index as u32)
    }

    pub fn add_image_from_bytes(
        &mut self,
        desc: &wgpu::TextureDescriptor,
        data: &[u8],
    ) -> ImageRef {
        let texture = self
            .device
            .create_texture_with_data(&self.queue, &desc, data);

        self.add_image_from_texture(texture, desc.size.clone())
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Do we need explicit cleanup?
    }
}

impl ContextBuilder {
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

    pub async fn build_offscreen(self) -> Context {
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

        Context {
            instance,
            surface: None,
            device,
            queue,
            targets: Vec::new(),
            images: Vec::new(),
            meshes: Vec::new(),
        }
    }

    pub async fn build<W: HasWindow>(self, window: &W) -> Context {
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
            width: size.x,
            height: size.y,
            alpha_mode,
            present_mode: surface_capabilities.present_modes[0],
            view_formats: vec![surface_format],
        };

        surface.configure(&device, &config);

        // @TODO adopt Ruffle's convention:
        // this is a FramebufferTarget
        let context = SurfaceContext {
            instance: surface,
            config,
        };

        Context {
            instance,
            surface: Some(context),
            device,
            queue,
            targets: Vec::new(),
            images: Vec::new(),
            meshes: Vec::new(),
        }
    }
}
