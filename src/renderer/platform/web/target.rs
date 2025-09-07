use crate::{RenderContext, Size, Target, TargetFrame, WindowTarget};
use std::convert::TryInto;
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

#[wasm_bindgen]
impl CanvasTarget {
    #[wasm_bindgen(js_name = "resize")]
    pub fn resize_js(&mut self, size: JsValue) -> Result<(), JsError> {
        let sz: Size = size
            .try_into()
            .map_err(|e: crate::error::ShaderError| JsError::new(&format!("{e}")))?;
        Target::resize(self, sz);
        Ok(())
    }

    #[wasm_bindgen(js_name = "size")]
    pub fn size_js(&self) -> Size {
        Target::size(self)
    }

    #[wasm_bindgen(js_name = "getImage")]
    pub fn get_image_js(&self) -> Result<js_sys::Uint8Array, JsError> {
        // Not currently supported for window/surface-backed targets in WASM
        Err(JsError::new(
            "getImage() is only supported for texture targets",
        ))
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

#[wasm_bindgen]
pub struct TextureTarget(crate::TextureTarget);

impl From<crate::TextureTarget> for TextureTarget {
    fn from(texture_target: crate::TextureTarget) -> Self {
        Self(texture_target)
    }
}

impl Target for TextureTarget {
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

#[wasm_bindgen]
impl TextureTarget {
    #[wasm_bindgen(js_name = "resize")]
    pub fn resize_js(&mut self, size: JsValue) -> Result<(), JsError> {
        let sz: Size = size
            .try_into()
            .map_err(|e: crate::error::ShaderError| JsError::new(&format!("{e}")))?;
        Target::resize(self, sz);
        Ok(())
    }

    #[wasm_bindgen(js_name = "size")]
    pub fn size_js(&self) -> Size {
        Target::size(self)
    }

    #[wasm_bindgen(js_name = "getImage")]
    pub fn get_image_js(&self) -> js_sys::Uint8Array {
        let data = Target::get_image(self);
        js_sys::Uint8Array::from(data.as_slice())
    }
}
