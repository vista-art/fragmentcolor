use crate::{
    app::window::{IsWindow, Window},
    renderer::{
        options::{DEVICE_LIMITS, POWER_PREFERENCE},
        target::{
            RenderTarget, RenderTargetCollection, RenderTargets, TargetId, TextureTarget,
            WindowTarget,
        },
        RenderPass, RendererOptions,
    },
    resources::{
        mesh::{MeshData, MeshId},
        texture::{Texture, TextureId},
        Resources,
    },
    scene::Scene,
};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub type Commands = Vec<wgpu::CommandBuffer>;

type Error = Box<dyn std::error::Error>;

pub(crate) trait RenderContext {
    fn read_resources(&self) -> Result<RwLockReadGuard<Resources>, Error>;
    fn write_resources(&self) -> Result<RwLockWriteGuard<Resources>, Error>;
    fn read_targets(&self) -> Result<RwLockReadGuard<RenderTargets>, Error>;
    fn write_targets(&self) -> Result<RwLockWriteGuard<RenderTargets>, Error>;
    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
}

/// 🎨 Draws things on the screen or on a texture
///
/// The Renderer is the link between the CPU world and the GPU world.
#[derive(Debug)]
pub(crate) struct Renderer {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    resources: Arc<RwLock<Resources>>,
    targets: Arc<RwLock<RenderTargets>>,
    pass: String, // @TODO support multiple render passes
}

impl RenderContext for Renderer {
    /// Returns a read lock to the Resources Database.
    ///
    /// # Errors
    /// If the Resources Manager is locked for writing, qcquiring this lock would cause
    /// a deadlock, so an error is returned. This function does not block the thread to
    /// wait for the lock to be available. It's up to the caller to decide what to do.
    fn read_resources(&self) -> Result<RwLockReadGuard<Resources>, Error> {
        if let Ok(lock) = self.resources.try_read() {
            Ok(lock)
        } else {
            Err("Cannot Read Renderer's Resources Database. Operation cancelled.".into())
        }
    }

    /// Locks the Resource Manager for writing and returns a write lock guard to it.
    ///
    /// # Errors
    /// If the Resources Manager is locked for writing, qcquiring this lock would cause
    /// a deadlock, so an error is returned. This function does not block the thread to
    /// wait for the lock to be available. It's up to the caller to decide what to do.
    fn write_resources(&self) -> Result<RwLockWriteGuard<Resources>, Error> {
        if let Ok(lock) = self.resources.try_write() {
            Ok(lock)
        } else {
            Err("Cannot Write to Renderer's Resources Database. Operation cancelled.".into())
        }
    }

    /// Returns a read lock to the Targets Database.
    ///
    /// # Errors
    /// If the Targets Database is locked for writing, qcquiring this lock would cause
    /// a deadlock, so an error is returned. This function does not block the thread to
    /// wait for the lock to be available. It's up to the caller to decide what to do.
    fn read_targets(&self) -> Result<RwLockReadGuard<RenderTargets>, Error> {
        if let Ok(lock) = self.targets.try_read() {
            Ok(lock)
        } else {
            Err("Cannot Read Renderer's Targets Database. Operation cancelled.".into())
        }
    }

    /// Locks the Targets Database for writing and returns a write lock guard to it.
    ///
    /// # Errors
    /// If the Targets Database is locked for writing, qcquiring this lock would cause
    /// a deadlock, so an error is returned. This function does not block the thread to
    /// wait for the lock to be available. It's up to the caller to decide what to do.
    fn write_targets(&self) -> Result<RwLockWriteGuard<RenderTargets>, Error> {
        if let Ok(lock) = self.targets.try_write() {
            Ok(lock)
        } else {
            Err("Cannot Write to Renderer's Targets Database. Operation cancelled.".into())
        }
    }

    /// Returns a reference to the GPU device.
    fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Returns a reference to the GPU queue.
    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

impl Renderer {
    /// Creates a new Offscreen Renderer instance.
    ///
    /// # Errors
    /// - If the Renderer is already initialized
    /// - If the GPU is not available
    pub async fn new_offscreen(options: RendererOptions) -> Result<Renderer, Error> {
        Renderer::new::<Window>(options, None).await
    }

    /// Creates a new Renderer instance.
    pub async fn new<'w, W: IsWindow>(
        options: RendererOptions,
        window: Option<&'w W>,
    ) -> Result<Renderer, Error> {
        if crate::app::RENDERER.get().is_some() {
            return Err("Renderer already initialized".into());
        }

        let pass = options.render_pass.clone();
        let (instance, adapter, device, queue, targets) =
            Internal::gpu_objects(options, window).await?;
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

    /// Registers a loaded mesh to the Resources Manager.
    ///
    /// This function takes a MeshData instance generated by the MeshBuilder
    /// after it uploads the raw mesh vertex and index buffers to the GPU.
    pub fn add_mesh(&self, mesh: MeshData) -> Result<MeshId, Error> {
        if let Ok(mut resources) = self.write_resources() {
            Ok(resources.add_mesh(mesh))
        } else {
            Err("Failed to acquire Resources Database lock. Mesh not created!".into())
        }
    }

    /// Removes a mesh from the Resources Manager.
    #[allow(dead_code)]
    pub fn remove_mesh(&self, id: &MeshId) -> Result<Option<MeshData>, Error> {
        if let Ok(mut resources) = self.write_resources() {
            Ok(resources.remove_mesh(id))
        } else {
            Err("Failed to acquire Resources Database Write lock. Mesh not deleted!".into())
        }
    }

    /// Registers a loaded texture to the Resources Manager.
    ///
    /// The texture is already loaded into the GPU at this point.
    /// This is an Internal function used by the Texture itself.
    pub fn add_texture(&self, texture: Texture) -> Result<TextureId, Error> {
        if let Ok(mut resources) = self.write_resources() {
            Ok(resources.add_texture(texture))
        } else {
            Err("Failed to acquire Resources Database Write lock. Texture not created!".into())
        }
    }

    /// Removes a texture from the Resources Manager.
    #[allow(dead_code)]
    pub fn remove_texture(&self, id: &TextureId) -> Result<Option<Texture>, Error> {
        if let Ok(mut resources) = self.write_resources() {
            Ok(resources.remove_texture(id))
        } else {
            Err("Failed to acquire Resources Database Read lock. Texture not Deleted!".into())
        }
    }

    /// Registers an OS Window or a Web Canvas element as a rendering target.
    ///
    /// This method expects the Window to implement the `IsWindow` trait,
    /// which allows the renderer to assign a unique Target ID to it.
    #[allow(dead_code)] // @TODO decide if this is necessary or if a Window could use it
    pub async fn add_winodw_target<W: IsWindow>(&self, window: W) -> Result<TargetId, Error> {
        let surface = Internal::surface(&self.instance, Some(&window))?;
        let target = Internal::window_target(&self.device, &self.adapter, &window, surface);
        if let Ok(mut targets) = self.write_targets() {
            Ok(targets.add(target))
        } else {
            Err("Failed to acquire Render Targets Database Write lock. Target not created!".into())
        }
    }

    /// Registers a Texture as a rendering target.
    pub fn add_texture_target(&self, texture: Texture) -> Result<TargetId, Error> {
        let target = RenderTarget::Texture(TextureTarget::from_texture(self, texture)?);
        if let Ok(mut targets) = self.write_targets() {
            Ok(targets.add(target))
        } else {
            Err("Failed to acquire Render Targets Database Write lock. Target not created!".into())
        }
    }

    /// Removes a rendering target from the renderer.
    pub fn remove_target(&self, id: &TargetId) -> Result<Option<RenderTarget>, Error> {
        if let Ok(mut targets) = self.write_targets() {
            Ok(targets.remove(id))
        } else {
            Err("Failed to acquire Render Targets Database Write lock. Target not deleted!".into())
        }
    }

    /// Where the magic starts! 🪄
    ///
    /// Selects a RenderPass to render a frame from the given Scene
    pub fn render(&self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        if self.pass == "solid" {
            return self.solid_renderpass(scene);
        }
        if self.pass == "toy" {
            return self.toy_renderpass(scene);
        }
        self.flat_renderpass(scene)
    }

    // Renders the Flat 2D render pass (for sprites and shapes)
    fn flat_renderpass(&self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let renderpass = crate::renderer::renderpass::Flat2D::new(self);

        self.draw(scene, renderpass)
    }

    // Renders the Solid 3D render pass (for simple 3D primitives)
    fn solid_renderpass(&self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let renderpass = crate::renderer::renderpass::Solid::new(
            &crate::renderer::renderpass::SolidConfig {
                cull_back_faces: true,
            },
            self,
        );

        self.draw(scene, renderpass)
    }

    // Renders the Shadertoy render pass (for a single fullscreen quad)
    fn toy_renderpass(&self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        let renderpass = crate::renderer::renderpass::Toy::new(self);

        self.draw(scene, renderpass)
    }

    // Where the magic happens! 🎨
    //
    // Renders a frame from the given Scene with the given RenderPass
    fn draw<P: RenderPass>(
        &self,
        scene: &Scene,
        mut renderpass: P,
    ) -> Result<(), wgpu::SurfaceError> {
        // Records the render commands in the GPU command buffer
        let (commands, frames) = renderpass.draw(scene.read_state())?;

        // Runs the commands (submit to GPU queue)
        self.queue.submit(commands);

        // Shows the rendered frames on the screen
        if let Ok(mut targets) = self.write_targets() {
            targets.present(frames);
        } else {
            log::warn!("Dropped Frame: Cannot present! Failed to acquire Render Targets Database Write lock.");
            return Err(wgpu::SurfaceError::Lost);
        };

        Ok(())
    }
}

// Helper static methods
struct Internal;
impl Internal {
    async fn gpu_objects<'w, W: IsWindow>(
        options: RendererOptions,
        window: Option<&'w W>,
    ) -> Result<
        (
            wgpu::Instance,
            wgpu::Adapter,
            wgpu::Device,
            wgpu::Queue,
            RenderTargets,
        ),
        Error,
    > {
        let panic_on_device_error = options.panic_on_error;
        let instance = wgpu::Instance::default();
        let (power_preference, force_fallback_adapter, limits) = Internal::parse_options(options);
        let surface = if let Ok(surface) = Internal::surface(&instance, window) {
            Some(surface)
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
                    features: wgpu::Features::empty(),
                    limits,
                    label: None,
                },
                None, // Trace path
            )
            .await?;

        if !panic_on_device_error {
            device.on_uncaptured_error(Box::new(|error| {
                log::error!("\n\n==== GPU error: ====\n\n{:#?}\n", error);
            }))
        }

        let targets = Internal::render_targets(&device, &adapter, (window, surface));

        Ok((instance, adapter, device, queue, targets))
    }

    fn parse_options(options: RendererOptions) -> (wgpu::PowerPreference, bool, wgpu::Limits) {
        let preference = options.power_preference;
        let limits = options.device_limits;
        let power_preference = POWER_PREFERENCE
            .get(&preference)
            .unwrap_or(&wgpu::PowerPreference::default())
            .to_owned();
        let device_limits = DEVICE_LIMITS
            .get(&limits)
            .unwrap_or(&wgpu::Limits::default())
            .to_owned();
        let force_fallback_adapter = options.force_software_rendering;

        (power_preference, force_fallback_adapter, device_limits)
    }

    fn surface<'w, W: IsWindow>(
        instance: &wgpu::Instance,
        window: Option<&'w W>,
    ) -> Result<wgpu::Surface, Error> {
        let window = window.ok_or("No Window is present. Skipping GPU surface creation...")?;
        Ok(unsafe { instance.create_surface(window) }?)
    }

    fn window_surface_pair<'w, W: IsWindow>(
        window_surface_pair: (Option<&'w W>, Option<wgpu::Surface>),
    ) -> Option<(&'w W, wgpu::Surface)> {
        Some((
            if let Some(window) = window_surface_pair.0 {
                window
            } else {
                return None;
            },
            if let Some(surface) = window_surface_pair.1 {
                surface
            } else {
                return None;
            },
        ))
    }

    fn render_targets<W: IsWindow>(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        ws_pair: (Option<&W>, Option<wgpu::Surface>),
    ) -> RenderTargets {
        let mut targets = RenderTargets::new();
        if let Some((window, surface)) = Self::window_surface_pair(ws_pair) {
            let target = Self::window_target(device, adapter, window, surface);
            targets.add(target);
        };

        targets
    }

    fn window_target<W: IsWindow>(
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        window: &W,
        surface: wgpu::Surface,
    ) -> RenderTarget {
        // The shader code assumes an sRGB surface texture. Using a different one
        // will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_capabilities = surface.get_capabilities(&adapter);
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
        };

        surface.configure(&device, &config);

        RenderTarget::Window(WindowTarget {
            id: window.id(),
            scaling_factor: window.scaling(),
            surface,
            config,
        })
    }
}
