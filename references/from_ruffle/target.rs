use std::fmt::Debug;
use winit::window::Window;

pub trait RenderTargetFrame: Debug {
    fn into_view(self) -> wgpu::TextureView;

    fn view(&self) -> &wgpu::TextureView;
}

pub trait RenderTarget: Debug + 'static {
    type Frame: RenderTargetFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);
    // WINDOW (change config)
    //   self.surface_config.width = width;
    //   self.surface_config.height = height;
    //   self.window_surface.configure(device, &self.surface_config);
    //
    // TEXTURE (creates a new texture)
    //   *self = TextureTarget::new(device, (width, height)).expect("Unable to resize texture target");

    fn format(&self) -> wgpu::TextureFormat;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    // WINDOW
    //   self.surface_config.{width, height, format}
    //
    // TEXTURE
    //   self.format
    //   self.size.width
    //   self.size.height

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SurfaceError>;
    // WINDOW
    //   let texture = self.window_surface.get_current_texture()?;
    //   let view = texture.texture.create_view(&Default::default());
    //   Ok(SwapChainTargetFrame { texture, view })
    //
    // TEXTURE
    //   Ok(
    //     TextureTargetFrame(
    //         self.texture.create_view(&Default::default()),
    //     )
    //   )

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
        frame: Self::Frame,
    ) -> wgpu::SubmissionIndex;
    // WINDOW
    //
    //     let index = queue.submit(command_buffers);
    //     frame.texture.present();
    //     index
    //
    // TEXTURE
    //
    // if let Some(TextureBufferInfo { buffer, copy_area }) = &self.buffer {
    //     let label = create_debug_label!("Render target transfer encoder");
    //     let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    //         label: label.as_deref(),
    //     });
    //     let (buffer, dimensions) = buffer.inner();
    //
    //     // THE IMPORTANT PART IS HERE:
    //     encoder.copy_texture_to_buffer(
    //         wgpu::ImageCopyTexture {
    //             texture: &self.texture,
    //             mip_level: 0,
    //             origin: wgpu::Origin3d {
    //                 x: copy_area.x_min,
    //                 y: copy_area.y_min,
    //                 z: 0,
    //             },
    //             aspect: wgpu::TextureAspect::All,
    //         },
    //         wgpu::ImageCopyBuffer {
    //             buffer,
    //             layout: wgpu::ImageDataLayout {
    //                 offset: 0,
    //                 bytes_per_row: Some(dimensions.padded_bytes_per_row),
    //                 rows_per_image: None,
    //             },
    //         },
    //         wgpu::Extent3d {
    //             width: copy_area.width(),
    //             height: copy_area.height(),
    //             depth_or_array_layers: 1,
    //         },
    //     );
    //
    //     queue.submit(command_buffers.into_iter().chain(Some(encoder.finish())))
    // } else {
    //     queue.submit(command_buffers)
    // }
}

pub struct SwapChainTarget {
    window_surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
}
pub struct SwapChainTargetFrame {
    texture: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}
// fn pub fn new(
//     surface: wgpu::Surface,
//     adapter: &wgpu::Adapter,
//     (width, height): (u32, u32),
//     device: &wgpu::Device,
// )
// WINDOW: let it select the surface

pub struct TextureTarget {
    pub size: wgpu::Extent3d,
    pub texture: Arc<wgpu::Texture>,
    pub format: wgpu::TextureFormat,
    pub buffer: Option<TextureBufferInfo>,
}

pub struct TextureTargetFrame(wgpu::TextureView);

// The texture target uses those things:
#[derive(Debug)]
pub enum MaybeOwnedBuffer {
    Borrowed(PoolEntry<wgpu::Buffer, BufferDimensions>, BufferDimensions),
    Owned(wgpu::Buffer, BufferDimensions),
}

impl MaybeOwnedBuffer {
    pub fn inner(&self) -> (&wgpu::Buffer, &BufferDimensions) {
        match &self {
            MaybeOwnedBuffer::Borrowed(entry, dimensions) => ((*entry).deref(), dimensions),
            MaybeOwnedBuffer::Owned(buffer, dimensions) => (buffer, dimensions),
        }
    }
}

#[derive(Debug)]
pub struct TextureBufferInfo {
    pub buffer: MaybeOwnedBuffer,
    pub copy_area: PixelRegion,
}
