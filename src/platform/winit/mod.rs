use crate::{FragmentColor, InitializationError, Renderer};
use std::sync::Arc;
use winit::window::Window;

pub mod target;
pub use target::*;

impl FragmentColor {
    pub async fn init(
        window: Arc<Window>,
    ) -> Result<(Renderer, WindowTarget), InitializationError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = crate::platform::all::request_adapter(&instance, Some(&surface)).await?;

        let (device, queue) = crate::platform::all::request_device(&adapter).await?;

        let size = window.as_ref().inner_size();
        let capabilities = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilities.formats[0].remove_srgb_suffix(),
            width: u32::max(size.width, 1),
            height: u32::max(size.height, 1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let target = WindowTarget::new(surface, config);
        let renderer = Renderer::init(device, queue);

        Ok((renderer, target))
    }
}
