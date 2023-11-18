use crate::{
    app::window::IsWindow,
    renderer::{
        options::{DEVICE_LIMITS, POWER_PREFERENCE},
        target::{RenderTargetCollection, Target, TargetId, Targets, WindowTarget},
        RenderOptions, RenderPass,
    },
    resources::{
        mesh::{MeshData, MeshId},
        texture::{Texture, TextureId},
        Resources,
    },
    scene::Scene,
    Window,
};
use std::{
    fs::File,
    io,
    path::Path,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use wgpu::util::DeviceExt;
use winit::window::WindowId;

pub type Commands = Vec<wgpu::CommandBuffer>;

type Error = Box<dyn std::error::Error>;
type WindowSurface = (WindowId, wgpu::Extent3d, wgpu::Surface);
type WindowSurfaces = Vec<WindowSurface>;

pub trait RenderContext {
    fn resources(&self) -> Arc<RwLock<Resources>>;
    fn read_resources(&self) -> RwLockReadGuard<Resources>;
    fn write_resources(&self) -> RwLockWriteGuard<Resources>;
    fn targets(&self) -> Arc<RwLock<Targets>>;
    fn read_targets(&self) -> RwLockReadGuard<Targets>;
    fn write_targets(&self) -> RwLockWriteGuard<Targets>;
    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
}

// @TODO describe the renderer
/// 🎨 Draws things on the screen or on a texture
///
/// The Renderer is the link between the CPU world and the GPU world.
///
/// Full Description TBD
#[derive(Debug)]
pub struct Renderer {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) pass: String, // @TODO support multiple render passes
    pub(crate) targets: Arc<RwLock<Targets>>,
    pub(crate) resources: Arc<RwLock<Resources>>,
}

impl RenderContext for Renderer {
    fn resources(&self) -> Arc<RwLock<Resources>> {
        self.resources.clone()
    }
    fn read_resources(&self) -> RwLockReadGuard<Resources> {
        self.resources
            .try_read()
            .expect("Could not get resources mutex lock")
    }
    fn write_resources(&self) -> RwLockWriteGuard<Resources> {
        self.resources
            .try_write()
            .expect("Could not get resources mutex lock")
    }
    fn targets(&self) -> Arc<RwLock<Targets>> {
        self.targets.clone()
    }
    fn read_targets(&self) -> RwLockReadGuard<Targets> {
        self.targets
            .try_read()
            .expect("Could not get targets mutex lock")
    }
    fn write_targets(&self) -> RwLockWriteGuard<Targets> {
        self.targets
            .try_write()
            .expect("Could not get targets mutex lock")
    }
    fn device(&self) -> &wgpu::Device {
        &self.device
    }
    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

impl Renderer {
    pub async fn new_offscreen(options: RenderOptions) -> Result<Renderer, Error> {
        Renderer::new::<Window>(options, vec![]).await
    }

    pub async fn new<'w, W: IsWindow>(
        options: RenderOptions,
        windows: Vec<&'w mut W>,
    ) -> Result<Renderer, Error> {
        if crate::app::RENDERER.get().is_some() {
            return Err("Renderer already initialized".into());
        }

        let pass = options.render_pass.clone();
        let (instance, adapter, device, queue, targets) =
            Internal::gpu_objects(options, windows).await?;
        let targets = Arc::new(RwLock::new(targets));
        let resources = Arc::new(RwLock::new(Resources::new()));

        Ok(Renderer {
            instance,
            adapter,
            device,
            queue,
            pass,
            targets,
            resources,
        })
    }

    pub fn add_mesh(&self, mesh: MeshData) -> MeshId {
        self.write_resources().add_mesh(mesh)
    }

    pub fn add_texture(&self, texture: wgpu::Texture) -> TextureId {
        let texture = Texture::from_wgpu_texture(&self, texture);
        self.write_resources().add_texture(texture)
    }

    /// Registers an OS Window or a Web Canvas element as a rendering target.
    /// This method expects the Window to implement the `IsWindow` trait,
    /// which allows the renderer to assign a unique Target ID to it.
    pub async fn add_target<W: IsWindow>(&self, window: W) -> Result<TargetId, Error> {
        let surface = Internal::surface(&self.instance, &window)?;
        let target = Internal::target(&self.device, &self.adapter, surface);
        Ok(self.write_targets().add(target))
    }

    /// Removes a rendering target from the renderer.
    pub fn remove_target(&self, id: &TargetId) -> Option<Target> {
        self.write_targets().remove(id)
    }

    /// Where the magic starts! 🪄
    ///
    /// Selects a RenderPass to render a frame from the given Scene
    pub fn render(&self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        if self.pass == "solid" {
            return self.render_pass_solid(scene);
        }
        self.render_pass_flat(scene)
    }

    // Renders the Flat 2D render pass (for sprites and shapes)
    fn render_pass_flat(&self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let pass = crate::renderer::renderpass::Flat2D::new(self);

        self.pass(scene, pass)
    }

    // Renders the Solid 3D render pass
    fn render_pass_solid(&self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let pass = crate::renderer::renderpass::Solid::new(
            &crate::renderer::renderpass::SolidConfig {
                cull_back_faces: true,
            },
            self,
        );

        self.pass(scene, pass)
    }

    // Where the magic happens! 🎨
    //
    // Renders a frame from the given Scene with the given RenderPass
    fn pass<P: RenderPass>(&self, scene: &Scene, mut pass: P) -> Result<(), wgpu::SurfaceError> {
        // Records the render commands in the GPU command buffer
        let (commands, frames) = pass.draw(scene.read_state())?;

        let targets = self.read_targets();
        targets.render(self, commands); // Runs the commands (submit to GPU queue)
        drop(targets);

        let mut targets = self.write_targets();
        targets.present(frames); // Shows the rendered frames on the screen
        drop(targets);

        Ok(())
    }

    // @TODO this logic exists in the Texture module;
    //       we should delegate all this part to it.
    pub fn load_image(&self, path_ref: impl AsRef<Path>) -> Result<TextureId, Error> {
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
                    // @FIXME ALL asserts and panics must go away and return a Result
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
                    // @FIXME ALL asserts and panics must go away and return a Result
                    ref other => panic!("Unsupported DDS FourCC {:?}", other),
                }
            } else {
                // @FIXME ALL asserts and panics must go away and return a Result
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
            // @FIXME ALL asserts and panics must go away and return a Result
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
                mip_level_count: 1,
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
        options: RenderOptions,
        windows: Vec<&'w mut W>,
    ) -> Result<
        (
            wgpu::Instance,
            wgpu::Adapter,
            wgpu::Device,
            wgpu::Queue,
            Targets,
        ),
        Error,
    > {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let (power_preference, force_fallback_adapter, limits) = Internal::parse_options(options);
        let surfaces = Internal::surfaces(&instance, &windows);

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

        Ok((instance, adapter, device, queue, targets))
    }

    fn parse_options(options: RenderOptions) -> (wgpu::PowerPreference, bool, wgpu::Limits) {
        let preference = options.power_preference;
        let limits = options.device_limits;
        let power_preference = POWER_PREFERENCE.get(&preference).unwrap().to_owned();
        let device_limits = DEVICE_LIMITS.get(&limits).unwrap().to_owned();
        let force_fallback_adapter = options.force_software_rendering;

        (power_preference, force_fallback_adapter, device_limits)
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
        let size = window.size().to_wgpu_size();
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
}
