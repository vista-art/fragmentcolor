use crate::{Size, renderer::HasDisplaySize};
use wgpu::rwh::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle};

pub struct HeadlessWindow {
    // Desired size for the fallback TextureTarget
    size: Size,
}

impl HasDisplaySize for HeadlessWindow {
    fn size(&self) -> Size {
        self.size
    }
}

/// Construct a headless window that can be passed to Renderer::create_target.
/// This will render using an offscreen Texture under the hood via the renderer's
/// own fallback path (no surface creation is attempted from here).
pub fn headless_window(size: impl Into<Size>) -> HeadlessWindow {
    HeadlessWindow { size: size.into() }
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
