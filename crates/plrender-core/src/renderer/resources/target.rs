use crate::renderer::{
    resources::{
        buffer::{Buffer, BufferSize, TextureBuffer},
        region::TextureRegion,
        texture::Texture,
    },
    Renderer,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::{fmt::Debug, sync::Arc};

type Error = Box<dyn std::error::Error>;

pub trait HasWindow: HasRawDisplayHandle + HasRawWindowHandle + RenderTarget {}

pub trait RenderTarget: Debug + 'static {
    fn size(&self) -> wgpu::Extent3d;
    fn aspect(&self) -> f32;
    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) -> Result<(), Error>;
    fn view(&self) -> Option<&wgpu::TextureView>;
    fn format(&self) -> wgpu::TextureFormat;
    fn sample_count(&self) -> u32;

    // fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SurfaceError>;

    // fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
    //     &self,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     command_buffers: I,
    //     frame: Self::Frame,
    // ) -> wgpu::SubmissionIndex;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetRef(pub u8);

#[derive(Debug)]
pub enum Target {
    //Window(WindowTarget),
    Texture(TextureTarget),
    Frame(FrameTarget),
}

#[derive(Debug)]
pub struct TextureTarget {
    pub texture: Arc<crate::Texture>,
    pub buffer: Option<TextureBuffer>,
}

#[derive(Debug)]
pub struct WindowTarget {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

#[derive(Debug)]
pub struct FrameTarget {
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub size: wgpu::Extent3d,
}

// @TODO separate impls instead of enums with enum-dispatch?
impl RenderTarget for Target {
    fn size(&self) -> wgpu::Extent3d {
        match self {
            Self::Texture(target) => target.texture.size,
            Self::Frame(frame) => frame.size,
        }
    }

    fn aspect(&self) -> f32 {
        let size = self.size();
        size.width as f32 / size.height as f32
    }

    fn resize(&mut self, renderer: &Renderer, size: wgpu::Extent3d) -> Result<(), Error> {
        match self {
            Self::Texture(_) => {
                let new_target = TextureTarget::new(renderer, size)?;
                *self = Target::Texture(new_target);
            }
            Self::Frame(_) => {}
        };

        Ok(())
    }

    fn format(&self) -> wgpu::TextureFormat {
        match self {
            Self::Texture(target) => target.texture.format,
            Self::Frame(frame) => frame.format,
        }
    }

    fn view(&self) -> Option<&wgpu::TextureView> {
        match self {
            Self::Texture(target) => Some(&target.texture.view),
            Self::Frame(frame) => Some(&frame.view),
        }
    }

    fn sample_count(&self) -> u32 {
        match self {
            Self::Texture(target) => target.texture.data.sample_count(),
            Self::Frame(_) => 1,
        }
    }
}

// @ TODO we reached a point where we can remove this now.
/// Parameters of a texture target that affect its pipeline compatibility.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetInfo {
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub aspect_ratio: f32,
}

impl TextureTarget {
    pub fn new(renderer: &Renderer, size: wgpu::Extent3d) -> Result<Self, Error> {
        let texture = Texture::create_target_texture(renderer, size);
        Self::from_texture(renderer, texture)
    }

    pub fn from_wgpu_texture(renderer: &Renderer, texture: wgpu::Texture) -> Self {
        let texture = Texture::from_wgpu_texture(renderer, texture);
        Self::from_texture(renderer, texture).unwrap()
    }

    pub fn from_texture(renderer: &Renderer, texture: Texture) -> Result<Self, Error> {
        let size = texture.size;
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
}
