use crate::{
    Size,
    renderer::{HasDisplaySize, SurfaceSource},
};

#[derive(Clone, Copy, Debug)]
pub struct HeadlessWindow {
    size: Size,
}

/// Construct a headless window that can be passed to Renderer::create_target.
/// This will render using an offscreen Texture under the hood, enabling examples
/// and doctests to run in CI or non-GUI environments.
pub fn headless_window(size: impl Into<Size>) -> HeadlessWindow {
    HeadlessWindow { size: size.into() }
}

/// Backwards-compatibility shim for older docs that referenced mock_window.
#[allow(dead_code)]
pub fn mock_window(size: impl Into<Size>) -> HeadlessWindow {
    headless_window(size)
}

impl HasDisplaySize for HeadlessWindow {
    fn size(&self) -> Size {
        self.size
    }
}

// Signal the Renderer to use the headless, texture-backed path.
impl SurfaceSource for HeadlessWindow {
    fn surface_handle(&self) -> Option<wgpu::SurfaceTarget<'static>> {
        None
    }
}
