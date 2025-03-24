use crate::{InitializationError, Renderer, WindowTarget};
use std::sync::Arc;
use winit::window::Window;

impl Renderer {
    pub async fn create_target(
        &self,
        window: Arc<Window>,
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
