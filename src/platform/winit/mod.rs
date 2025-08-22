use crate::{FragmentColor, InitializationError, Renderer, WindowTarget};
use std::sync::Arc;
use winit::window::Window;

impl FragmentColor {
    pub async fn init(
        window: Arc<Window>,
    ) -> Result<(Renderer, WindowTarget), InitializationError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = crate::platform::all::request_adapter(&instance, Some(&surface)).await?;
        let (device, queue) = crate::platform::all::request_device(&adapter).await?;
        let size = wgpu::Extent3d {
            width: window.inner_size().width,
            height: window.inner_size().height,
            depth_or_array_layers: 1,
        };
        let config = crate::platform::all::configure_surface(&device, &adapter, &surface, &size);

        let target = WindowTarget::new(surface, config);
        let renderer = Renderer::init(device, queue);

        Ok((renderer, target))
    }
}
