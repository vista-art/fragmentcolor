use crate::{RenderContext, Size, SurfaceError, Target, TargetFrame};
use lsp_doc::lsp_doc;
use std::sync::Arc;

#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug)]
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

impl crate::target::TargetInternal for WindowTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, SurfaceError> {
        let frame = self.acquire_frame()?;
        Ok(Box::new(frame))
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

    /// Reading back from a presentable surface needs `COPY_SRC` on the
    /// swapchain config, which we don't request yet. Returns an empty
    /// `Vec` for now; render to a [`TextureTarget`] instead when readback
    /// is required (CI image comparison, screenshot tooling).
    #[lsp_doc("docs/api/targets/target/get_image.md")]
    async fn get_image(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl WindowTarget {
    /// Try to acquire a frame; on Lost/Outdated, reconfigure and retry once.
    fn acquire_frame(&self) -> Result<WindowFrame, SurfaceError> {
        match crate::target::surface_texture_from(self.surface.get_current_texture()) {
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
                match err {
                    SurfaceError::Lost | SurfaceError::Outdated => {
                        // Reconfigure with the last known good config and retry once.
                        self.surface.configure(&self.context.device, &self.config);
                        let surface_texture = crate::target::surface_texture_from(
                            self.surface.get_current_texture(),
                        )?;
                        let view = surface_texture
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        Ok(WindowFrame {
                            surface_texture,
                            format: self.config.format,
                            view,
                        })
                    }
                    _ => Err(err),
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

    fn present(self: Box<Self>, queue: &wgpu::Queue) {
        queue.present(self.surface_texture);
    }
}
