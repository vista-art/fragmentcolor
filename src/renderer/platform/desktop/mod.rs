use crate::{InitializationError, Renderer, WindowTarget};

#[cfg(not(feature = "winit"))]
impl Renderer {
    pub async fn create_target(
        &'_ self,
        window: impl Into<wgpu::SurfaceTarget<'static>> + Clone,
        width: u32,
        height: u32,
    ) -> Result<WindowTarget, InitializationError> {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let (context, surface, config) = self.create_surface(window.clone(), size).await?;

        Ok(WindowTarget::new(context, surface, config))
    }
}

#[cfg(feature = "winit")]
impl Renderer {
    pub async fn create_target(
        &'_ self,
        window: Arc<winit::Window>,
    ) -> Result<WindowTarget, InitializationError> {
        let size = wgpu::Extent3d {
            width: window.inner_size().width,
            height: window.inner_size().height,
            depth_or_array_layers: 1,
        };
        let (context, surface, config) = self.create_surface(window.clone(), size).await?;

        Ok(WindowTarget::new(context, surface, config))
    }
}
