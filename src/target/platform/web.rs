#![cfg(wasm)]

use crate::error::WebError;
use crate::texture::TextureError;
use crate::{RenderContext, ShaderError, Size, Target, TargetFrame, TextureTarget, WindowTarget};
use lsp_doc::lsp_doc;
use parking_lot::Mutex;
use std::convert::TryInto;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

/// The window target on the web, exported to JS as `WindowTarget`.
/// `CanvasTarget` stays available in JS as an alias of the same class.
#[wasm_bindgen(js_name = "WindowTarget")]
#[derive(Clone)]
pub struct CanvasTarget {
    inner: Arc<Mutex<WindowTarget>>,
}

// `impl_fc_kind!` can't name the renamed JS class, so the brand is written out.
impl crate::FcKind for CanvasTarget {
    const KIND: &'static str = "WindowTarget";
}

#[wasm_bindgen(js_class = "WindowTarget")]
impl CanvasTarget {
    #[doc(hidden)]
    #[wasm_bindgen(getter, js_name = "__fc_kind")]
    pub fn js_fc_kind(&self) -> String {
        <Self as crate::FcKind>::KIND.into()
    }
}

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

#[wasm_bindgen(js_class = "WindowTarget")]
impl CanvasTarget {
    #[wasm_bindgen(js_name = "resize")]
    #[lsp_doc("docs/api/targets/window_target/resize.md")]
    pub fn resize_js(&mut self, size: &JsValue) -> Result<(), WebError> {
        let sz = Size::try_from(size)?;
        Target::resize(self, sz)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "size")]
    #[lsp_doc("docs/api/targets/window_target/size.md")]
    pub fn size_js(&self) -> Size {
        Target::size(self)
    }

    #[wasm_bindgen(js_name = "getImage")]
    #[lsp_doc("docs/api/targets/window_target/get_image.md")]
    pub async fn get_image_js(&self) -> Result<js_sys::Uint8Array, TextureError> {
        let data = Target::get_image(self).await?;
        Ok(js_sys::Uint8Array::from(data.as_slice()))
    }
}

impl crate::target::FrameSource for CanvasTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
        self.inner.lock().get_current_frame()
    }

    fn owner(&self) -> Option<Arc<RenderContext>> {
        self.inner.lock().owner()
    }
}

impl Target for CanvasTarget {
    fn size(&self) -> Size {
        self.inner.lock().size()
    }

    fn resize(&mut self, size: impl Into<Size>) -> Result<(), TextureError> {
        self.inner.lock().resize(size)
    }

    async fn get_image(&self) -> Result<Vec<u8>, TextureError> {
        Err(crate::target::window::unreadable())
    }
}

#[wasm_bindgen]
impl TextureTarget {
    #[wasm_bindgen(js_name = "resize")]
    #[lsp_doc("docs/api/targets/texture_target/resize.md")]
    pub fn resize_js(&mut self, size: &JsValue) -> Result<(), WebError> {
        let size: Size = size.try_into()?;
        self.resize(size)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "size")]
    #[lsp_doc("docs/api/targets/texture_target/size.md")]
    pub fn size_js(&self) -> Size {
        self.size()
    }

    // A plain function, so the call takes its start before it returns the promise.
    #[wasm_bindgen(js_name = "getImage", unchecked_return_type = "Promise<Uint8Array>")]
    #[lsp_doc("docs/api/targets/texture_target/get_image.md")]
    pub fn get_image_js(&self) -> js_sys::Promise {
        let start = crate::core::Start::now(Some(&self.context));
        let target = self.clone();
        crate::error::defer(async move {
            let pixels = target.read_image_from(start).await?;
            Ok(js_sys::Uint8Array::from(pixels.as_slice()).into())
        })
    }

    /// Get a sampleable Texture handle for binding in a shader uniform.
    /// Mirrors the native `TextureTarget::texture()` method.
    #[wasm_bindgen(js_name = "texture")]
    #[lsp_doc("docs/api/targets/texture_target/texture.md")]
    pub fn texture_js(&self) -> crate::texture::Texture {
        self.texture()
    }
}

crate::impl_js_bridge!(CanvasTarget, ShaderError, WindowTarget);
crate::impl_js_bridge!(TextureTarget, ShaderError);
