use crate::{Size, renderer::HasDisplaySize};
use wgpu::rwh::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle};

#[doc(hidden)]
pub struct HeadlessWindow {
    // Desired size for the fallback TextureTarget
    size: Size,
}

impl HasDisplaySize for HeadlessWindow {
    fn size(&self) -> Size {
        self.size
    }
}

impl HeadlessWindow {
    /// A window of `size` with no surface, for the doctests and tests that need one.
    /// `Renderer::create_target` draws it into an offscreen texture of that size.
    pub fn new(size: impl Into<Size>) -> Self {
        Self { size: size.into() }
    }
}

impl HasWindowHandle for HeadlessWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}

impl HasDisplayHandle for HeadlessWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Err(HandleError::Unavailable)
    }
}
