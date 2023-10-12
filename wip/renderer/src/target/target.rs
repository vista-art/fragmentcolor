use std::fmt::Debug;
use winit::window::Window;

pub trait RenderTargetFrame: Debug {
    fn into_view(self) -> wgpu::TextureView;

    fn view(&self) -> &wgpu::TextureView;
}

pub trait RenderTarget: Debug + 'static {
    type Frame: RenderTargetFrame;

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);

    fn format(&self) -> wgpu::TextureFormat;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn get_next_texture(&mut self) -> Result<Self::Frame, wgpu::SurfaceError>;

    fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_buffers: I,
        frame: Self::Frame,
    ) -> wgpu::SubmissionIndex;
}

struct Target<T: RenderTarget> {
    target: T,
    clear_color: Option<Color>,
    window: Option<Window>,
    callback: Option<Box<dyn FnMut()>>,
}

impl Target {
    fn new_framebuffer() {}
    fn new_offscreen() {}
}
