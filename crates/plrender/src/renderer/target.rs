use crate::renderer::{
    resources::{
        buffer::{Buffer, BufferSize, TextureBuffer},
        region::TextureRegion,
        texture::Texture,
    },
    Renderer,
};
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use winit::window::WindowId;

type Error = Box<dyn std::error::Error>;
type Commands = Vec<wgpu::CommandBuffer>;
type SubmissionIndex = wgpu::SubmissionIndex;

pub trait HasSize {
    fn size(&self) -> wgpu::Extent3d;
    fn aspect(&self) -> f32;
}

pub struct Frame {
    surface_texture: Option<wgpu::SurfaceTexture>,
    pub view: wgpu::TextureView,
}

impl Frame {
    fn present(self) {
        if self.surface_texture.is_some() {
            self.surface_texture.unwrap().present();
        }
    }
}
pub trait RenderTarget: Debug + 'static + HasSize {
    fn format(&self) -> wgpu::TextureFormat;
    fn sample_count(&self) -> u32;
    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) -> Result<(), Error>;
    fn next_frame(&self) -> Result<Frame, wgpu::SurfaceError>;
    fn submit(&self, renderer: &Renderer, commands: Commands, frame: Frame) -> SubmissionIndex;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetId {
    Texture(u8),
    Window(WindowId),
}

#[derive(Debug)]
pub enum Target {
    Texture(TextureTarget),
    Window(WindowTarget),
}

#[derive(Debug)]
pub struct Targets {
    texture_count: u8,
    pub targets: HashMap<TargetId, Target>,
}

impl Targets {
    pub fn new() -> Self {
        Self {
            texture_count: 0,
            targets: HashMap::new(),
        }
    }

    pub fn add(&mut self, target: Target) -> TargetId {
        let id = match target {
            Target::Texture(_) => {
                self.texture_count += 1;
                TargetId::Texture(self.texture_count)
            }
            Target::Window(ref target) => TargetId::Window(target.id),
        };

        self.targets.insert(id, target);

        id
    }

    pub fn get(&self, id: &TargetId) -> Option<&Target> {
        self.targets.get(id)
    }

    pub fn get_mut(&mut self, id: &TargetId) -> Option<&mut Target> {
        self.targets.get_mut(id)
    }

    pub fn remove(&mut self, id: &TargetId) -> Option<Target> {
        self.targets.remove(id)
    }

    pub fn all(&self) -> impl Iterator<Item = &Target> {
        self.targets.values()
    }
}

#[derive(Debug)]
pub struct TextureTarget {
    pub texture: Arc<Texture>,
    pub buffer: Option<TextureBuffer>,
}

#[derive(Debug)]
pub struct WindowTarget {
    pub id: WindowId,
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

impl HasSize for Target {
    fn size(&self) -> wgpu::Extent3d {
        match self {
            Self::Texture(target) => target.texture.size,
            Self::Window(target) => target.size(),
        }
    }
    fn aspect(&self) -> f32 {
        let size = self.size();
        size.width as f32 / size.height as f32
    }
}

impl RenderTarget for Target {
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

    // Maybe this is not the right abstraction if we want multiple targets of different types.
    // Queue.submit() happens once per frame, but this method will be called for every target.
    // They should add things to the queue, but not submit it.
    fn submit(&self, renderer: &Renderer, commands: Commands, frame: Frame) -> SubmissionIndex {
        match self {
            // Texture does things BEFORE submit (adds copy command to CommandBuffer)
            Target::Texture(target) => target.submit(renderer, commands),
            // Window does things AFTER submit (present to the screen)
            //
            Target::Window(window) => window.present(renderer, commands, frame),
        }
    }
}

impl WindowTarget {
    fn size(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.config.width,
            height: self.config.height,
            depth_or_array_layers: 1,
        }
    }

    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&renderer.device, &self.config)
    }

    // NOTE: window.present() happens once per target,
    //       but queue.submit() should happen once per frame.
    fn present(&self, renderer: &Renderer, commands: Commands, frame: Frame) -> SubmissionIndex {
        let index = renderer.queue.submit(commands);
        frame.present();
        index
    }
}

impl TextureTarget {
    pub fn new(renderer: &Renderer, size: wgpu::Extent3d) -> Result<Self, Error> {
        let texture = Texture::create_target_texture(renderer, size);
        Self::from_texture(renderer, texture)
    }

    pub fn from_wgpu_texture(renderer: &Renderer, texture: wgpu::Texture) -> Result<Self, Error> {
        let texture = Texture::from_wgpu_texture(renderer, texture);
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

        Ok(Self {
            texture: Arc::new(texture),
            buffer: Some(TextureBuffer {
                buffer: Buffer {
                    size: buffer_size,
                    buffer,
                },
                clip_area: TextureRegion::for_whole_size(size.width, size.height),
            }),
        })
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

    fn submit(&self, renderer: &Renderer, commands: Commands) -> SubmissionIndex {
        if let Some(TextureBuffer { buffer, clip_area }) = &self.buffer {
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
                        x: clip_area.x_min,
                        y: clip_area.y_min,
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
                    width: clip_area.width(),
                    height: clip_area.height(),
                    depth_or_array_layers: 1,
                },
            );

            let commands = commands.into_iter().chain(Some(encoder.finish()));
            renderer.queue.submit(commands)
        } else {
            renderer.queue.submit(commands)
        }
    }
}
