use crate::{
    app::events::{Callback, Event},
    app::window::IsWindow,
    components,
    components::Camera,
    math::geometry::Quad,
    renderer::{Commands, Renderer},
    resources::{
        buffer::{Buffer, BufferSize, TextureBuffer},
        texture::{Texture, TextureId},
    },
    scene::{ObjectId, SceneObject},
};
use std::{
    collections::{
        hash_map::{Values, ValuesMut},
        HashMap,
    },
    fmt::Debug,
};
use winit::window::WindowId;

type Error = Box<dyn std::error::Error>;
pub type RenderedFrames = Vec<(TargetId, Frame)>;
pub trait Dimensions {
    fn size(&self) -> Quad;
    fn aspect(&self) -> f32;
}

#[derive(Debug)]
pub struct Frame {
    surface_texture: Option<wgpu::SurfaceTexture>,
    pub view: wgpu::TextureView,
}

impl Frame {
    pub fn should_present(&self) -> bool {
        self.surface_texture.is_some()
    }

    pub fn present(self) {
        if self.should_present() {
            self.surface_texture.unwrap().present();
        }
    }
}

pub trait RenderTarget: Debug + 'static + Dimensions {
    fn id(&self) -> TargetId;
    fn format(&self) -> wgpu::TextureFormat;
    fn sample_count(&self) -> u32;
    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) -> Result<(), Error>;
    fn next_frame(&self) -> Result<Frame, wgpu::SurfaceError>;
    fn prepare_render(&self, renderer: &Renderer, commands: &mut Commands);
    fn present(&mut self, frame: Frame);
}

pub trait RenderTargetCollection: Debug + 'static {
    fn len(&self) -> usize;
    fn add(&mut self, target: Target) -> TargetId;
    fn get(&self, id: &TargetId) -> Option<&Target>;
    fn get_mut(&mut self, id: &TargetId) -> Option<&mut Target>;
    fn remove(&mut self, id: &TargetId) -> Option<Target>;
    fn all(&self) -> Values<TargetId, Target>;
    fn all_mut(&mut self) -> ValuesMut<TargetId, Target>;
    fn present(&mut self, frames: RenderedFrames);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetId {
    Texture(TextureId),
    Window(WindowId),
}

#[derive(Debug)]
pub enum Target {
    Texture(TextureTarget),
    Window(WindowTarget),
}

/// Describes how a target should be rendered.
#[derive(Clone, Debug)]
pub struct RenderTargetDescription {
    /// The Id of the target to render to.
    ///
    /// If it's a Window, the Id can be obtained with
    /// `my_window.id()`. If it's a Texture, the Id
    /// will be returned by the Renderer when the
    /// texture is created.
    pub target_id: TargetId,

    /// The size of the target in pixels.
    pub target_size: Quad,

    /// The camera to use when rendering to this target.
    ///
    /// If None, the Scene will assign the first available
    /// camera to this target. If there is no camera in
    /// the Scene, the Scene will create a default 2D
    /// camera (orthographic projection).
    pub camera_id: Option<ObjectId>,

    /// The color to use when clearing the target.
    ///
    /// Defaults to Transparent. If the OS does not
    /// support transparent windows, this will be
    /// ignored and users will see the background
    /// color of the window.
    ///
    /// On Web, this draws a transparent canvas.
    pub clear_color: components::Color,

    /// Reserved for future use (unimplemented).
    ///
    /// The Renderer will draw on top of this target
    /// without clearing its previous contents.
    pub background_image: Option<TextureId>,

    /// The region of the target to render to.
    ///
    /// Defaults to the full target.
    ///
    /// Users can use this to render to a portion of
    /// the target. For example, to render the Eye
    /// cameras on top of the World camera.
    pub clip_region: Quad,

    /// Callback function to run right before rendering.
    ///
    /// This is useful for updating uniforms, and syncing
    /// the Scene state with the main rendering loop.
    pub before_render: Option<Callback<Event>>,

    /// Callback function to run after rendering.
    ///
    /// Returns a &[u8] with the contents of the target.
    pub after_render: Option<Callback<Event>>,
}

impl RenderTargetDescription {
    pub fn new(target_id: TargetId, target_size: Quad) -> Self {
        Self {
            target_id,
            target_size,
            camera_id: None,
            clear_color: components::Color::default(),
            background_image: None,
            clip_region: target_size,
            before_render: None,
            after_render: None,
        }
    }

    pub fn from_window<W: IsWindow>(window: &W) -> Self {
        Self::new(TargetId::Window(window.id()), window.size())
    }

    pub fn from_texture(texture: &Texture) -> Self {
        Self::new(TargetId::Texture(texture.id), texture.size())
    }

    pub fn attach_cammera(self, camera: &SceneObject<Camera>) -> Result<Self, Error> {
        let camera_id = if let Some(camera_id) = camera.id() {
            camera_id
        } else {
            return Err("Camera must be added to a Scene before having a Render Target".into());
        };
        Ok(self.set_camera_id(camera_id))
    }

    pub fn set_camera(self, camera: &SceneObject<Camera>) -> Self {
        self.attach_cammera(camera)
            .expect("Camera is not in a Scene")
    }

    pub fn set_camera_id(mut self, camera_id: ObjectId) -> Self {
        self.camera_id = Some(camera_id);
        self
    }

    pub fn set_clear_color(mut self, clear_color: components::Color) -> Self {
        self.clear_color = clear_color;
        self
    }

    pub fn set_background_image(mut self, background_image: TextureId) -> Self {
        self.background_image = Some(background_image);
        self
    }

    pub fn set_clip_region(mut self, clip_region: Quad) -> Self {
        self.clip_region = clip_region;
        self
    }

    pub fn before_render(mut self, callback: Callback<Event>) -> Self {
        self.before_render = Some(callback);
        self
    }

    pub fn after_render(mut self, callback: Callback<Event>) -> Self {
        self.after_render = Some(callback);
        self
    }
}

#[derive(Debug)]
pub struct Targets {
    pub targets: HashMap<TargetId, Target>,
}

impl Targets {
    pub fn new() -> Self {
        Self {
            targets: HashMap::new(),
        }
    }
}

impl RenderTargetCollection for Targets {
    fn add(&mut self, target: Target) -> TargetId {
        let id = match target {
            Target::Texture(ref target) => TargetId::Texture(target.texture.id),
            Target::Window(ref window) => TargetId::Window(window.id),
        };

        self.targets.insert(id, target);

        id
    }

    fn get(&self, id: &TargetId) -> Option<&Target> {
        self.targets.get(id)
    }

    fn get_mut(&mut self, id: &TargetId) -> Option<&mut Target> {
        self.targets.get_mut(id)
    }

    fn remove(&mut self, id: &TargetId) -> Option<Target> {
        self.targets.remove(id)
    }

    fn all(&self) -> Values<TargetId, Target> {
        self.targets.values()
    }

    fn all_mut(&mut self) -> ValuesMut<TargetId, Target> {
        self.targets.values_mut()
    }

    fn len(&self) -> usize {
        self.targets.len()
    }

    fn present(&mut self, frames: RenderedFrames) {
        for (target_id, frame) in frames {
            let target = self.targets.get_mut(&target_id).unwrap();
            target.present(frame);
        }
    }
}

#[derive(Debug)]
pub struct TextureTarget {
    pub texture: Texture,
    pub buffer: Option<TextureBuffer>,
}

#[derive(Debug)]
pub struct WindowTarget {
    pub id: WindowId,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

impl Dimensions for Target {
    fn size(&self) -> Quad {
        match self {
            Self::Texture(target) => Quad::from_wgpu_size(target.texture.size),
            Self::Window(target) => target.size(),
        }
    }
    fn aspect(&self) -> f32 {
        self.size().aspect()
    }
}

impl RenderTarget for Target {
    fn id(&self) -> TargetId {
        match self {
            Self::Texture(target) => TargetId::Texture(target.texture.id),
            Self::Window(window) => TargetId::Window(window.id),
        }
    }

    fn format(&self) -> wgpu::TextureFormat {
        match self {
            Self::Texture(target) => target.texture.format,
            Self::Window(window) => window.config.format,
        }
    }

    fn sample_count(&self) -> u32 {
        match self {
            Self::Texture(target) => target.texture.data.sample_count(),
            Self::Window(_) => 1,
        }
    }

    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) -> Result<(), Error> {
        match self {
            Self::Texture(_) => {
                let new_target = TextureTarget::new(renderer, size)?;
                *self = Target::Texture(new_target);
            }
            Self::Window(window) => window.resize(renderer, size),
        };

        Ok(())
    }

    fn next_frame(&self) -> Result<Frame, wgpu::SurfaceError> {
        // @TODO handle multiple cameras in the same target (cache frame, second pass).
        //       needs a guard here to check if the frame has been presented yet
        match self {
            Self::Texture(target) => Ok(Frame {
                surface_texture: None,
                view: target.texture.data.create_view(&Default::default()),
            }),
            Self::Window(window) => {
                let frame = window.surface.get_current_texture()?;
                let view = frame.texture.create_view(&Default::default());
                Ok(Frame {
                    surface_texture: Some(frame),
                    view,
                })
            }
        }
    }

    fn prepare_render(&self, renderer: &Renderer, commands: &mut Commands) {
        match self {
            Target::Texture(target) => target.copy_texture_to_buffer(renderer, commands),
            Target::Window(_) => {}
        }
    }

    fn present(&mut self, frame: Frame) {
        match self {
            Target::Window(_) => frame.present(),
            _ => {}
        }
    }
}

impl WindowTarget {
    fn size(&self) -> Quad {
        Quad::from_dimensions(self.config.width, self.config.height)
    }

    /// Rebuilds the swap chain with the new Window size
    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&renderer.device, &self.config)
    }
}

impl TextureTarget {
    pub fn new(renderer: &Renderer, size: wgpu::Extent3d) -> Result<Self, Error> {
        let texture = Texture::create_target_texture(size);
        Self::from_texture(renderer, texture)
    }

    pub fn from_texture(renderer: &Renderer, texture: Texture) -> Result<Self, Error> {
        let size = texture.size;
        Self::validate(renderer, size)?;

        let buffer_size = BufferSize::new(size.width as usize, size.height as usize);
        let buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Render target buffer"),
            size: buffer_size.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let target = Self {
            texture,
            buffer: Some(TextureBuffer {
                buffer: Buffer {
                    size: buffer_size,
                    buffer,
                },
                clip_region: Quad::from_dimensions(size.width, size.height),
            }),
        };

        Ok(target)
    }

    fn validate(renderer: &Renderer, size: wgpu::Extent3d) -> Result<(), Error> {
        if size.width > renderer.device.limits().max_texture_dimension_2d
            || size.height > renderer.device.limits().max_texture_dimension_2d
            || size.width < 1
            || size.height < 1
        {
            return Err(format!(
                "Texture target cannot be smaller than 1 or larger than {}px on either dimension (requested {} x {})",
                renderer.device.limits().max_texture_dimension_2d,
                size.width,
                size.height,
            )
            .into());
        }

        Ok(())
    }

    fn copy_texture_to_buffer(&self, renderer: &Renderer, commands: &mut Commands) {
        if let Some(TextureBuffer {
            buffer,
            clip_region,
        }) = &self.buffer
        {
            let mut encoder =
                renderer
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render target transfer encoder"),
                    });
            let Buffer { buffer, size } = buffer;

            encoder.copy_texture_to_buffer(
                // Our rendered texture
                wgpu::ImageCopyTexture {
                    texture: &self.texture.data,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: clip_region.x_min,
                        y: clip_region.y_min,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                // The destination buffer
                wgpu::ImageCopyBuffer {
                    buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(size.padded_bytes_per_row),
                        rows_per_image: None,
                    },
                },
                // Clip area
                wgpu::Extent3d {
                    width: clip_region.width(),
                    height: clip_region.height(),
                    depth_or_array_layers: 1,
                },
            );

            commands.append(&mut vec![encoder.finish()])
        }
    }
}
