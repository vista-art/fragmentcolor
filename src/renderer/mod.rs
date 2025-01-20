pub mod options;
pub mod pass;
pub mod target;

pub use {options::*, pass::*, target::*};

use crate::{
    math::geometry::Quad,
    renderer::{options::POWER_PREFERENCE, Pass, RendererOptions},
    resources::{
        texture::{Texture, TextureId},
        Resources,
    },
    MeshData, MeshId, Scene,
};
use std::sync::{Arc, RwLock};

pub type Commands = Vec<wgpu::CommandBuffer>;

type Error = Box<dyn std::error::Error>;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub trait Window: HasDisplayHandle + HasWindowHandle {
    fn id(&self) -> TextureId;
    fn size(&self) -> Quad;
    fn request_redraw(&self);
}

/// 🎨 Draws things on the screen or on a texture
///
/// The Renderer is the link between the CPU world and the GPU world.
pub struct Renderer {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) resources: Arc<RwLock<Resources>>,
    pub(crate) passes: Vec<Box<dyn Pass>>,
}

unsafe impl Sync for Renderer {}

impl Renderer {
    /// Creates a new Renderer instance.
    pub(crate) async fn new<W: Window + Sync>(
        options: RendererOptions,
        window: Option<&W>,
    ) -> Result<Renderer, Error> {
        let (instance, adapter, device, queue) = Self::gpu_objects(options, window).await?;

        let resources = Resources::new();
        let resources = Arc::new(RwLock::new(resources));

        Ok(Renderer {
            instance,
            adapter,
            device,
            queue,
            resources,
            passes: Vec::new(),
        })
    }

    pub fn add_mesh(&self, _mesh: MeshData) -> Result<MeshId, Error> {
        // FIXME
        Ok(MeshId(0))
    }

    /// Creates a new target Texture from a given size.
    pub fn create_target_texture(&self, size: Quad) -> Result<Texture, Error> {
        Ok(Texture::create_destination_texture(
            self,
            size.to_wgpu_size(),
        )?)
    }

    // /// Registers an OS Window or a Web Canvas element as a rendering target.
    // ///
    // /// This method expects the Window to implement the `Window` trait,
    // /// which allows the renderer to assign a unique Target ID to it.
    // pub(crate) fn add_winodw_target<W: Window>(&self, window: &W) -> Result<TextureId, Error> {
    //     let surface = Self::surface(&self.instance, Some(window))?;
    //     let target = Self::window_target(&self.device, &self.adapter, window, surface);
    //     if let Ok(mut targets) = self.write_targets() {
    //         Ok(targets.add(target))
    //     } else {
    //         Err(
    //             "Failed to acquire Render Targets Database Write lock. Window Target not created!"
    //                 .into(),
    //         )
    //     }
    // }

    // /// Registers a Texture as a rendering target.
    // pub(crate) fn add_texture_target(&self, texture: Texture) -> Result<TextureId, Error> {
    //     let target = RenderTarget::Texture(TextureTarget::from_texture(self, texture)?);

    //     let mut targets = self
    //         .targets
    //         .write()
    //         .expect("Failed to acquire Render Targets Database Write lock");
    //     let target_id = targets.add(target);

    //     Ok(target_id)
    // }

    // /// Removes a rendering target from the renderer.
    // pub(crate) fn remove_target(&self, id: &TextureId) -> Result<Option<RenderTarget>, Error> {
    //     if let Ok(mut targets) = self.write_targets() {
    //         Ok(targets.remove(id))
    //     } else {
    //         Err("Failed to acquire Render Targets Database Write lock. Target not deleted!".into())
    //     }
    // }

    /// Where the magic happens! 🎨
    ///
    /// Renders a frame from the given Scene with the given RenderPass
    pub(crate) fn render<P: Pass>(
        &self,
        _scene: &Scene,
        mut _renderpass: P,
    ) -> Result<(), wgpu::SurfaceError> {
        let _ = _scene;
        // FIXME

        // Records the render commands in the GPU command buffer
        // let (commands, frames) =
        //     renderpass.draw(self, scene, scene.first_camera().unwrap(), self.targets)?;

        // // Runs the commands (submit to GPU queue)
        // self.queue.submit(commands);

        // // Shows the rendered frames on the screen
        // if let Ok(mut targets) = self.write_targets() {
        //     targets.present(frames);
        // } else {
        //     log::warn!("Dropped Frame: Cannot present! Failed to acquire Render Targets Database Write lock.");
        //     return Err(wgpu::SurfaceError::Lost);
        // };

        Ok(())
    }

    async fn gpu_objects<W: Window + Sync>(
        options: RendererOptions,
        window: Option<&W>,
    ) -> Result<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue), Error> {
        let panic_on_device_error = options.panic_on_error;
        let instance = wgpu::Instance::default();
        let (power_preference, force_fallback_adapter, limits) = Self::parse_options(options);

        let surface = if let Some(window) = window {
            instance.create_surface(window).ok()
        } else {
            None
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                force_fallback_adapter,
                compatible_surface: surface.as_ref(),
            })
            .await
            .ok_or("Failed to find an appropriate GPU adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None, // Trace path
            )
            .await?;

        if !panic_on_device_error {
            device.on_uncaptured_error(Box::new(|error| {
                log::error!("\n\n==== GPU error: ====\n\n{:#?}\n", error);
            }))
        }

        Ok((instance, adapter, device, queue))
    }

    fn parse_options(options: RendererOptions) -> (wgpu::PowerPreference, bool, wgpu::Limits) {
        let preference = options.power_preference;
        let limits = options.device_limits;
        let power_preference = POWER_PREFERENCE
            .get(&preference)
            .unwrap_or(&wgpu::PowerPreference::default())
            .to_owned();
        let device_limits = wgpu::Limits::default();
        let force_fallback_adapter = options.force_software_rendering;

        (power_preference, force_fallback_adapter, device_limits)
    }

    fn window_target<'w, W: Window>(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        window: &'w W,
        surface: wgpu::Surface<'w>,
    ) -> RenderTarget<'w> {
        // The shader code assumes an sRGB surface texture. Using a different one
        // will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_capabilities = surface.get_capabilities(adapter);
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
            width: window.size().width(),
            height: window.size().height(),
            alpha_mode,
            present_mode: surface_capabilities.present_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 3,
        };

        surface.configure(device, &config);

        RenderTarget::Window(WindowTarget {
            id: window.id(),
            scaling_factor: 1.0, //window.scaling(),
            surface,
            config,
        })
    }
}
