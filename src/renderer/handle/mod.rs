use wgpu::rwh::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle};

#[derive(Clone, Copy)]
pub struct WindowHandles<'window> {
    pub window_handle: WindowHandle<'window>,
    pub display_handle: DisplayHandle<'window>,
}

unsafe impl<'window> Send for WindowHandles<'window> {}
unsafe impl<'window> Sync for WindowHandles<'window> {}

impl<'window> HasWindowHandle for WindowHandles<'window> {
    fn window_handle(&self) -> Result<WindowHandle<'window>, HandleError> {
        Ok(self.window_handle)
    }
}

impl<'window> HasDisplayHandle for WindowHandles<'window> {
    fn display_handle(&self) -> Result<DisplayHandle<'window>, HandleError> {
        Ok(self.display_handle)
    }
}
