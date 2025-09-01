use crate::{RenderContext, Size, Target, TargetFrame, WindowTarget};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct CanvasTarget(WindowTarget);

impl CanvasTarget {
    pub(crate) fn new(
        context: Arc<RenderContext>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self(WindowTarget::new(context, surface, config))
    }
}

impl Target for CanvasTarget {
    fn size(&self) -> Size {
        self.0.size()
    }

    fn resize(&mut self, size: impl Into<Size>) {
        self.0.resize(size);
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.0.get_current_frame()
    }
}
