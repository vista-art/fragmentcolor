use crate::Error;
use std::mem::size_of;

#[derive(Debug)]
pub struct SwapChainTarget {
    window_surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
}

#[derive(Debug)]
pub struct SwapChainTargetFrame {
    texture: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

impl RenderTargetFrame for SwapChainTargetFrame {
    fn into_view(self) -> wgpu::TextureView {
        self.view
    }

    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

impl SwapChainTarget {
    pub fn new(
        surface: wgpu::Surface,
        adapter: &wgpu::Adapter,
        (width, height): (u32, u32),
        device: &wgpu::Device,
    ) -> Self {
        // Ideally we want to use an RGBA non-sRGB surface format, because Flash colors and
        // blending are done in sRGB space -- we don't want the GPU to adjust the colors.
        // Some platforms may only support an sRGB surface, in which case we will draw to an
        // intermediate linear buffer and then copy to the sRGB surface.
        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities
            .formats
            .iter()
            .find(|format| {
                matches!(
                    format,
                    wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
                )
            })
            .or_else(|| capabilities.formats.first())
            .copied()
            // No surface (rendering to texture), default to linear RBGA.
            .unwrap_or(wgpu::TextureFormat::Rgba8Unorm);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: &[format],
        };
        surface.configure(device, &surface_config);
        Self {
            surface_config,
            window_surface: surface,
        }
    }
}

impl RenderTarget for SwapChainTarget {
    type Frame = SwapChainTargetFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.window_surface.configure(device, &self.surface_config);
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    fn width(&self) -> u32 {
        self.surface_config.width
    }

    fn height(&self) -> u32 {
        self.surface_config.height
    }

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SurfaceError> {
        let texture = self.window_surface.get_current_texture()?;
        let view = texture.texture.create_view(&Default::default());
        Ok(SwapChainTargetFrame { texture, view })
    }

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
        frame: Self::Frame,
    ) -> wgpu::SubmissionIndex {
        let index = queue.submit(command_buffers);
        frame.texture.present();
        index
    }
}
