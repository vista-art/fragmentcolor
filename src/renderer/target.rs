use crate::{
    components::{self},
    math::geometry::Quad,
    renderer::{Commands, Renderer},
    resources::{
        buffer::{Buffer, BufferSize, TextureBuffer},
        texture::{Texture, TextureId},
    },
    scene::ObjectId,
};
use image::Rgba;
use std::fmt::Debug;

type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetId(pub u32);

pub trait Dimensions {
    fn size(&self) -> Quad;
    fn aspect(&self) -> f32;
    fn scaling(&self) -> f32;
}

/// Interface to add a RenderTarget to a Scene.
///
/// Any object that implements this trait can be iserted in the
/// Scene with `scene.target(my_target)`.
pub trait DescribesTarget {
    fn describe_target(&self) -> Result<RenderTargetDescription, Error>;
}

pub(crate) trait IsRenderTarget: Debug + Dimensions {
    fn id(&self) -> TextureId;
    fn format(&self) -> wgpu::TextureFormat;
    fn sample_count(&self) -> u32;
    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) -> Result<(), Error>;
    fn next_frame(&self) -> Result<Frame, wgpu::SurfaceError>;
    fn prepare_render(&self, renderer: &Renderer, commands: &mut Commands);
    fn present(&mut self, frame: Frame);
}

/// Describes how a RenderTarget should be rendered.
///
/// This objects maps a Scene Camera to a RenderTarget. Both the
/// Camera and the RenderTarget must exist.
#[derive(Clone, Debug)]
pub struct RenderTargetDescription {
    /// The Id of the RenderTarget to render to.
    ///
    /// If it's a Window, the Id can be obtained with
    /// `my_window.id()`. If it's a Texture, the Id
    /// is be returned when the texture is created:
    /// `let texture_id = renderer.add_texture(texture)`
    pub target_texture_id: TextureId,

    /// The camera to use when rendering to this target.
    ///
    /// If None, the Scene will assign the first available
    /// camera to this target. If there is no camera in
    /// the Scene, the Scene will create a default 2D
    /// camera (orthographic projection).
    pub camera_id: ObjectId,

    /// The size of the target in pixels.
    ///
    /// Defaults to the Target's full size.
    pub target_size: Quad,

    /// The color to use when clearing the target.
    ///
    /// Defaults to Transparent. If the OS does not
    /// support transparent windows, this will be
    /// ignored and users will see the background
    /// color of the window.
    ///
    /// On Web, this draws a transparent canvas.
    pub clear_color: components::Color,
}

/// Allow a TargetDescription to describe itself so it can
/// be used as a manual configuration for a RenderTarget
/// if the user wants to have t
impl DescribesTarget for RenderTargetDescription {
    fn describe_target(&self) -> Result<RenderTargetDescription, Error> {
        let target = self.clone();
        Ok(target)
    }
}

impl RenderTargetDescription {
    pub fn new(target_texture_id: TextureId, target_size: Quad, camera_id: ObjectId) -> Self {
        Self {
            target_texture_id,
            target_size,
            camera_id,
            clear_color: components::Color::default(),
        }
    }

    pub fn set_camera_id(&mut self, camera_id: ObjectId) -> &mut Self {
        self.camera_id = camera_id;
        self
    }

    pub fn set_clear_color(&mut self, clear_color: components::Color) -> &mut Self {
        self.clear_color = clear_color;
        self
    }
}

#[derive(Debug)]
pub(crate) struct Frame {
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

#[derive(Debug)]
pub(crate) enum RenderTarget<'w> {
    Texture(TextureTarget),
    Window(WindowTarget<'w>),
}

#[derive(Debug)]
pub(crate) struct TextureTarget {
    pub texture: Texture,
    pub buffer: Option<TextureBuffer>,
}

#[derive(Debug)]
pub(crate) struct WindowTarget<'w> {
    pub id: TextureId,
    pub scaling_factor: f32,
    pub surface: wgpu::Surface<'w>,
    pub config: wgpu::SurfaceConfiguration,
}

impl Dimensions for RenderTarget<'_> {
    fn size(&self) -> Quad {
        match self {
            Self::Texture(target) => Quad::from_wgpu_size(target.texture.size),
            Self::Window(target) => target.size(),
        }
    }
    fn aspect(&self) -> f32 {
        self.size().aspect()
    }
    fn scaling(&self) -> f32 {
        match self {
            Self::Texture(_) => 1.0,
            Self::Window(target) => target.scaling(),
        }
    }
}

impl IsRenderTarget for RenderTarget<'_> {
    fn id(&self) -> TextureId {
        match self {
            Self::Texture(target) => target.texture.id,
            Self::Window(window) => window.id,
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
                *self = RenderTarget::Texture(new_target);
            }
            Self::Window(window) => window.resize(renderer, size),
        };

        Ok(())
    }

    fn next_frame(&self) -> Result<Frame, wgpu::SurfaceError> {
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
        if let RenderTarget::Texture(target) = self {
            target.copy_texture_to_buffer(renderer, commands)
        }
    }

    fn present(&mut self, frame: Frame) {
        if let RenderTarget::Window(_) = self {
            frame.present()
        }
    }
}

impl<'w> WindowTarget<'w> {
    fn size(&self) -> Quad {
        Quad::from_size(self.config.width, self.config.height)
    }

    fn scaling(&self) -> f32 {
        self.scaling_factor
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
        let texture = renderer.create_target_texture(Quad::from_size(size.width, size.height))?;
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
                inner: Buffer {
                    size: buffer_size,
                    buffer,
                },
                clip_region: Quad::from_size(size.width, size.height),
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
        if let Some(TextureBuffer { inner, clip_region }) = &self.buffer {
            let mut encoder =
                renderer
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render target transfer encoder"),
                    });
            let Buffer { buffer, size } = inner;

            encoder.copy_texture_to_buffer(
                // Our rendered texture
                wgpu::ImageCopyTexture {
                    texture: &self.texture.data,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: clip_region.min_x,
                        y: clip_region.min_y,
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

    // @TODO call this at some point.
    #[allow(dead_code)]
    pub async fn get_rendered_frame_bytes(&self, renderer: &Renderer) -> Result<Vec<u8>, Error> {
        if let Some(texture_buffer) = &self.buffer {
            let output_buffer = &texture_buffer.inner.buffer;

            // We need to scope the mapping variables so that we can unmap the buffer
            let rendered_image = {
                let buffer_slice = output_buffer.slice(..);

                // NOTE: We have to create the mapping THEN device.poll()
                // before await the future. Otherwise the application will freeze.
                let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();

                buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                    sender.send(result).unwrap();
                });

                renderer.device.poll(wgpu::Maintain::Wait);

                let output_buffer_data = if let Some(received) = receiver.receive().await {
                    if received.is_ok() {
                        buffer_slice.get_mapped_range()
                    } else {
                        return Err("Failed to map texture buffer".into());
                    }
                } else {
                    return Err("Failed to read texture buffer".into());
                };

                let buffer = image::ImageBuffer::<Rgba<u8>, _>::from_raw(
                    self.texture.size.width,
                    self.texture.size.height,
                    output_buffer_data,
                )
                .unwrap();

                buffer.to_vec()
            };

            output_buffer.unmap();

            Ok(rendered_image)
        } else {
            Err("No texture buffer available to copy from".into())
        }
    }
}
