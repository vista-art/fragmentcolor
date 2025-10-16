#![cfg(wasm)]

use crate::{RenderContext, ShaderError, Size, Target, TargetFrame, TextureTarget, WindowTarget};
use lsp_doc::lsp_doc;
use parking_lot::Mutex;
use std::convert::TryInto;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct CanvasTarget {
    inner: Arc<Mutex<WindowTarget>>,
}

crate::impl_fc_kind!(CanvasTarget, "CanvasTarget");

impl CanvasTarget {
    pub(crate) fn new(
        context: Arc<RenderContext>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(WindowTarget::new(context, surface, config))),
        }
    }
}

#[wasm_bindgen]
impl CanvasTarget {
    #[wasm_bindgen(js_name = "resize")]
    #[lsp_doc("docs/api/targets/window_target/resize.md")]
    pub fn resize_js(&mut self, size: &JsValue) -> Result<(), JsError> {
        let sz: Size = size
            .try_into()
            .map_err(|e: crate::size::error::SizeError| JsError::new(&format!("{e}")))?;
        Target::resize(self, sz);
        Ok(())
    }

    #[wasm_bindgen(js_name = "size")]
    #[lsp_doc("docs/api/targets/window_target/size.md")]
    pub fn size_js(&self) -> Size {
        Target::size(self)
    }

    #[wasm_bindgen(js_name = "getImage")]
    #[lsp_doc("docs/api/targets/window_target/get_image.md")]
    pub fn get_image_js(&self) -> Result<js_sys::Uint8Array, JsError> {
        Err(JsError::new(
            "getImage() is only supported for texture targets",
        ))
    }
}

impl Target for CanvasTarget {
    fn size(&self) -> Size {
        self.inner.lock().size()
    }

    fn resize(&mut self, size: impl Into<Size>) {
        self.inner.lock().resize(size);
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.inner.lock().get_current_frame()
    }

    fn get_image(&self) -> Vec<u8> {
        vec![]
    }
}

#[wasm_bindgen]
impl TextureTarget {
    #[wasm_bindgen(js_name = "resize")]
    pub fn resize_js(&mut self, size: &JsValue) -> Result<(), JsError> {
        let size: Size = size.try_into()?;
        self.resize(size);
        Ok(())
    }

    #[wasm_bindgen(js_name = "size")]
    pub fn size_js(&self) -> Size {
        self.size()
    }

    #[wasm_bindgen(js_name = "getImage")]
    pub async fn get_image_js(&self) -> js_sys::Uint8Array {
        let data = self.get_image_async().await;
        js_sys::Uint8Array::from(data.as_slice())
    }
}

crate::impl_js_bridge!(CanvasTarget, ShaderError);
crate::impl_js_bridge!(TextureTarget, ShaderError);
