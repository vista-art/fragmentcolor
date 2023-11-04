use crate::scene::{camera::Camera, Scene};
use crate::target::{IsWindow, Target, TargetId, Targets, WindowContainer, WindowTarget};
use crate::{
    renderer::{
        options::POWER_PREFERENCE,
        resources::{
            mesh::{Mesh, MeshId},
            Resources,
        },
        texture::{Texture, TextureId},
        RenderOptions, RenderPass,
    },
    target::Windows,
};
use std::{
    fs::File,
    io,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};
use wgpu::util::DeviceExt;
use winit::window::WindowId;

type Error = Box<dyn std::error::Error>;
type WindowSurface = (WindowId, wgpu::Extent3d, wgpu::Surface);
type WindowSurfaces = Vec<WindowSurface>;

pub trait RenderContext {
    fn resources(&self) -> MutexGuard<'_, Resources>;
    fn targets(&self) -> MutexGuard<'_, Targets>;
    fn windows(&self) -> MutexGuard<'_, Windows>;
    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
}

#[derive(Debug)]
pub struct Renderer {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) windows: Arc<Mutex<Windows>>,
    pub(crate) targets: Arc<Mutex<Targets>>,
    pub(crate) resources: Arc<Mutex<Resources>>,
}

impl RenderContext for Renderer {
    fn resources(&self) -> MutexGuard<'_, Resources> {
        self.resources
            .try_lock()
            .expect("Could not get resources mutex lock")
    }
    fn targets(&self) -> MutexGuard<'_, Targets> {
        self.targets
            .try_lock()
            .expect("Could not get targets mutex lock")
    }
    fn windows(&self) -> MutexGuard<'_, Windows> {
        self.windows
            .try_lock()
            .expect("Could not get windows mutex lock")
    }
    fn device(&self) -> &wgpu::Device {
        &self.device
    }
    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

impl Renderer {
    pub async fn new<'w, W: IsWindow>(options: RenderOptions<'w, W>) -> Result<Renderer, Error> {
        let (instance, adapter, device, queue, windows, targets) =
            Internal::gpu_objects(options).await?;
        let targets = Arc::new(Mutex::new(targets));
        let windows = Arc::new(Mutex::new(windows));
        let resources = Arc::new(Mutex::new(Resources::new()));

        Ok(Renderer {
            instance,
            adapter,
            device,
            queue,
            windows,
            targets,
            resources,
        })
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshId {
        self.resources().add_mesh(mesh)
    }

    pub fn add_texture(&mut self, texture: wgpu::Texture) -> TextureId {
        let texture = Texture::from_wgpu_texture(&self, texture);
        self.resources().add_texture(texture)
    }

    /// Registers an OS Window or a Web Canvas element as a rendering target.
    /// This method expects the Window to implement the `IsWindow` trait,
    /// which allows the renderer to assign a unique Target ID to it.
    pub async fn add_target<W: IsWindow>(&mut self, window: W) -> Result<TargetId, Error> {
        let surface = Internal::surface(&self.instance, &window)?;
        let target = Internal::target(&self.device, &self.adapter, surface);
        Ok(self.targets().add(target))
    }

    // @TODO consider removing the arguments for this method.
    // The renderer will hold them internally, the user can
    // set them, and just call renderer.render() without arguments
    pub fn render<P: RenderPass>(
        &mut self,
        pass: &mut P, // how to abstract this from my user?
        scene: &Scene,
        camera: &Camera,
    ) -> Result<(), Error> {
        // removed a lot of things before that line
        // which assumed that we would always render to a window.

        // let frame = target.next_frame()?;

        // Will you delegate the targets callbacks to the RenderPass?
        // it makes sense, since it creates and holds the command encoder and command buffer
        //
        // doing it outside the pass would make reading difficult (splitting context)

        //let resources = self.resources()?;

        // @TODO consider acquiring the frame here

        pass.draw(scene, camera, self);

        // @TODO2 ... and submitting it here
        // (so this frame management won't be responsibility of the RenderPass.
        //                                      the renderpass would only draw)

        // @TODO multiple passes?
        // for target in self.targets {
        // ...
        //}

        Ok(())
    }

    // @TODO this logic exists in the Texture module;
    //       we should delegate all this part to it.
    pub fn load_image(&mut self, path_ref: impl AsRef<Path>) -> Result<TextureId, Error> {
        let path = path_ref.as_ref();
        let extension = path.extension().ok_or("No file extension detected")?;

        let image_format = image::ImageFormat::from_extension(extension).ok_or(format!(
            "Unrecognized image extension: {:?}",
            path.extension()
        ))?;

        let label = path.display().to_string();
        let file = File::open(path)?;

        let mut buf_reader = io::BufReader::new(file);

        let texture = if image_format == image::ImageFormat::Dds {
            let dds = ddsfile::Dds::read(&mut buf_reader)?;

            log::info!("Header {:?}", dds.header);
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

            texture
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

            texture
        };

        Ok(self.add_texture(texture))
    }

    // @TODO delegate to Texture impl
    pub fn add_texture_from_bytes(
        &mut self,
        desc: &wgpu::TextureDescriptor,
        data: &[u8],
    ) -> TextureId {
        let texture = self
            .device
            .create_texture_with_data(&self.queue, &desc, data);

        self.add_texture(texture)
    }
}

// Helper static methods
struct Internal;
impl Internal {
    async fn gpu_objects<'w, W: IsWindow>(
        options: RenderOptions<'w, W>,
    ) -> Result<
        (
            wgpu::Instance,
            wgpu::Adapter,
            wgpu::Device,
            wgpu::Queue,
            Windows,
            Targets,
        ),
        Error,
    > {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let (power_preference, force_fallback_adapter, window_list, limits) =
            Internal::parse_options(options);
        let surfaces = Internal::surfaces(&instance, &window_list);
        let windows = Internal::windows(window_list);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                force_fallback_adapter,
                compatible_surface: surfaces.first().map(|(_, _, surface)| surface),
            })
            .await
            .ok_or("Failed to find an appropriate GPU adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits,
                    label: None,
                },
                None, // Trace path
            )
            .await?;

        let targets = Internal::targets(&device, &adapter, surfaces);

        Ok((instance, adapter, device, queue, windows, targets))
    }

    fn parse_options<'w, W: IsWindow>(
        options: RenderOptions<'w, W>,
    ) -> (wgpu::PowerPreference, bool, Vec<&'w mut W>, wgpu::Limits) {
        let preference = options.power_preference.unwrap_or("high-performance");
        let power_preference = POWER_PREFERENCE.get(preference).unwrap().to_owned();
        let force_fallback_adapter = options.force_software_rendering.unwrap_or(false);
        let window_targets = match options.targets {
            Some(targets) => targets,
            None => Vec::new(),
        };

        // @TODO this should come from the RenderOptions
        //       and the JS wrapper would set it to "webgl2"
        let device_limits = if cfg!(wasm) {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };

        (
            power_preference,
            force_fallback_adapter,
            window_targets,
            device_limits,
        )
    }

    fn surfaces<'w, W: IsWindow>(
        instance: &wgpu::Instance,
        window_list: &Vec<&'w mut W>,
    ) -> WindowSurfaces {
        window_list
            .into_iter()
            .filter_map(|window| {
                let surface = Self::surface(instance, *window).ok()?;
                Some(surface)
            })
            .collect()
    }

    fn surface<'w, W: IsWindow>(
        instance: &wgpu::Instance,
        window: &'w W,
    ) -> Result<WindowSurface, Error> {
        let surface = unsafe { instance.create_surface(window) }?;
        let size = window.size();
        let id = window.id();
        Ok((id, size, surface))
    }

    fn targets(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        surfaces: WindowSurfaces,
    ) -> Targets {
        let mut targets = Targets::new();
        for surface in surfaces.into_iter() {
            let target = Internal::target(device, adapter, surface);
            targets.add(target);
        }

        targets
    }

    fn target(device: &wgpu::Device, adapter: &wgpu::Adapter, surface: WindowSurface) -> Target {
        let (id, size, surface) = surface;

        let surface_capabilities = surface.get_capabilities(&adapter);

        // The shader code assumes an sRGB surface texture. Using a different one
        // will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let format = surface_capabilities
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
            format,
            width: size.width,
            height: size.height,
            alpha_mode,
            present_mode: surface_capabilities.present_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Target::Window(WindowTarget {
            id,
            surface,
            config,
        })
    }

    fn windows<'w, W: IsWindow>(window_list: Vec<&'w mut W>) -> Windows {
        let mut windows = Windows::new();
        for window in window_list {
            windows.insert(window.id(), window.instance());
        }

        windows
    }
}