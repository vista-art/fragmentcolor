use crate::{Size, renderer::HasDisplaySize};
use wgpu::rwh::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle};

#[derive(Clone, Copy, Debug)]
pub struct MockWindow {
    size: Size,
}

/// Construct a mock window that can be passed to Renderer::create_target for documentation
/// and compile-time examples. This type implements HasDisplaySize and the modern handle traits
/// but its handle accessors return an error, so attempting to actually create a real Surface
/// from it will fail at runtime. Prefer TextureTarget for headless rendering in tests.
pub fn mock_window(size: impl Into<Size>) -> MockWindow {
    MockWindow { size: size.into() }
}

impl HasDisplaySize for MockWindow {
    fn size(&self) -> Size {
        self.size
    }
}

impl HasWindowHandle for MockWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl HasDisplayHandle for MockWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}
