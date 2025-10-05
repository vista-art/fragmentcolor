use crate::{RenderContext, Size, Target, TargetFrame};
use lsp_doc::lsp_doc;
use std::sync::Arc;

#[lsp_doc("docs/api/targets/window_target/window_target.md")]
pub struct WindowTarget {
    pub(crate) context: Arc<RenderContext>,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl WindowTarget {
    pub(crate) fn new(
        context: Arc<RenderContext>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self {
            context,
            surface,
            config,
        }
    }
}

impl Target for WindowTarget {
    #[lsp_doc("docs/api/targets/target/size.md")]
    fn size(&self) -> Size {
        [self.config.width, self.config.height].into()
    }

    #[lsp_doc("docs/api/targets/target/resize.md")]
    fn resize(&mut self, size: impl Into<Size>) {
        let size = size.into();
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.context.device, &self.config);
    }

    #[lsp_doc("docs/api/targets/target/hidden/get_current_frame.md")]
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        let frame = self.acquire_frame()?;
        Ok(Box::new(frame))
    }

    #[lsp_doc("docs/api/targets/target/get_image.md")]
    fn get_image(&self) -> Vec<u8> {
        // Reading back from a presentable surface is not portable across backends,
        // especially on WebGPU/WebGL. Prefer rendering to a TextureTarget when
        // readback is required (e.g., for CI image comparison).
        Vec::new()
    }
}

impl WindowTarget {
    /// Try to acquire a frame; on Lost/Outdated, reconfigure and retry once.
    fn acquire_frame(&self) -> Result<WindowFrame, wgpu::SurfaceError> {
        match self.surface.get_current_texture() {
            Ok(surface_texture) => {
                let view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                Ok(WindowFrame {
                    surface_texture,
                    format: self.config.format,
                    view,
                })
            }
            Err(err) => {
                use wgpu::SurfaceError::*;
                match err {
                    Lost | Outdated => {
                        // Reconfigure with the last known good config and retry once.
                        self.surface.configure(&self.context.device, &self.config);
                        let surface_texture = self.surface.get_current_texture()?;
                        let view = surface_texture
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        Ok(WindowFrame {
                            surface_texture,
                            format: self.config.format,
                            view,
                        })
                    }
                    Timeout | OutOfMemory | Other => Err(err),
                }
            }
        }
    }
}

struct WindowFrame {
    surface_texture: wgpu::SurfaceTexture,
    format: wgpu::TextureFormat,
    view: wgpu::TextureView,
}

impl TargetFrame for WindowFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    fn present(self: Box<Self>) {
        self.surface_texture.present();
    }
}
