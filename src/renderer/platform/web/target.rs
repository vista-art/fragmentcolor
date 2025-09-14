use crate::{RenderContext, Size, Target, TargetFrame, WindowTarget};
use parking_lot::Mutex;
use std::convert::TryInto;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct CanvasTarget {
    inner: Arc<Mutex<WindowTarget>>,
}

crate::impl_tryfrom_owned_via_ref!(TextureTarget, wasm_bindgen::JsValue, crate::error::ShaderError);

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
    pub fn resize_js(&mut self, size: &JsValue) -> Result<(), JsError> {
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
#[derive(Clone)]
pub struct TextureTarget(Arc<Mutex<crate::TextureTarget>>);

impl From<crate::TextureTarget> for TextureTarget {
    fn from(texture_target: crate::TextureTarget) -> Self {
        Self(Arc::new(Mutex::new(texture_target)))
    }
}

impl Target for TextureTarget {
    fn size(&self) -> Size {
        self.0.lock().size()
    }

    fn resize(&mut self, size: impl Into<Size>) {
        self.0.lock().resize(size);
    }

    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.0.lock().get_current_frame()
    }

    fn get_image(&self) -> Vec<u8> {
        self.0.lock().get_image()
    }
}

#[wasm_bindgen]
impl TextureTarget {
    #[wasm_bindgen(js_name = "resize")]
    pub fn resize_js(&mut self, size: &JsValue) -> Result<(), JsError> {
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

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for CanvasTarget {
    type Error = crate::error::ShaderError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::Reflect;
        use wasm_bindgen::convert::RefFromWasmAbi;
        let key = wasm_bindgen::JsValue::from_str("__wbg_ptr");
        let ptr = Reflect::get(value, &key).map_err(|_| {
            crate::error::ShaderError::WasmError("Missing __wbg_ptr on CanvasTarget".into())
        })?;
        let id = ptr.as_f64().ok_or_else(|| {
            crate::error::ShaderError::WasmError("Invalid __wbg_ptr for CanvasTarget".into())
        })? as u32;
        let anchor: <CanvasTarget as RefFromWasmAbi>::Anchor =
            unsafe { <CanvasTarget as RefFromWasmAbi>::ref_from_abi(id) };
        Ok(anchor.clone())
    }
}

crate::impl_tryfrom_owned_via_ref!(CanvasTarget, wasm_bindgen::JsValue, crate::error::ShaderError);

#[cfg(wasm)]
impl TryFrom<&wasm_bindgen::JsValue> for TextureTarget {
    type Error = crate::error::ShaderError;

    fn try_from(value: &wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        use js_sys::Reflect;
        use wasm_bindgen::convert::RefFromWasmAbi;
        let key = wasm_bindgen::JsValue::from_str("__wbg_ptr");
        let ptr = Reflect::get(value, &key).map_err(|_| {
            crate::error::ShaderError::WasmError("Missing __wbg_ptr on TextureTarget".into())
        })?;
        let id = ptr.as_f64().ok_or_else(|| {
            crate::error::ShaderError::WasmError("Invalid __wbg_ptr for TextureTarget".into())
        })? as u32;
        let anchor: <TextureTarget as RefFromWasmAbi>::Anchor =
            unsafe { <TextureTarget as RefFromWasmAbi>::ref_from_abi(id) };
        Ok(anchor.clone())
    }
}
