use crate::{
    Frame, Mesh, Pass, Renderer, RendererError, Shader, Size, Texture, TextureInput, TextureTarget,
    target::CanvasTarget,
};
use js_sys::Array;
use lsp_doc::lsp_doc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

pub enum Canvas {
    Html(web_sys::HtmlCanvasElement),
    Offscreen(web_sys::OffscreenCanvas),
}

impl Canvas {
    pub fn size(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width(),
            height: self.height(),
            depth_or_array_layers: 1,
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            Self::Html(canvas) => canvas.width(),
            Self::Offscreen(canvas) => canvas.width(),
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Self::Html(canvas) => canvas.height(),
            Self::Offscreen(canvas) => canvas.height(),
        }
    }
}

impl From<web_sys::HtmlCanvasElement> for Canvas {
    fn from(canvas: web_sys::HtmlCanvasElement) -> Self {
        Self::Html(canvas)
    }
}

impl From<web_sys::OffscreenCanvas> for Canvas {
    fn from(canvas: web_sys::OffscreenCanvas) -> Self {
        Self::Offscreen(canvas)
    }
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/core/renderer/new.md")]
    pub fn new_js() -> Self {
        Self::new()
    }

    #[wasm_bindgen(js_name = "createTarget")]
    #[lsp_doc("docs/api/core/renderer/create_target.md")]
    pub async fn create_target_js(&self, canvas: JsValue) -> Result<CanvasTarget, JsError> {
        let canvas = if let Some(canvas) = canvas.dyn_ref::<web_sys::HtmlCanvasElement>() {
            Canvas::Html(canvas.clone())
        } else if let Ok(canvas) = canvas.dyn_into::<web_sys::OffscreenCanvas>() {
            Canvas::Offscreen(canvas)
        } else {
            return Err(JsError::new("Failed to convert input to Canvas"));
        };

        let size = canvas.size();
        let (context, surface, config) = match canvas {
            Canvas::Html(canvas) => {
                let target = wgpu::SurfaceTarget::Canvas(canvas);
                self.create_surface(target, size).await?
            }
            Canvas::Offscreen(canvas) => {
                let target = wgpu::SurfaceTarget::OffscreenCanvas(canvas);
                self.create_surface(target, size).await?
            }
        };

        Ok(CanvasTarget::new(context, surface, config))
    }

    /// JS: Create a Texture from bytes, URL, or a CSS selector/HTMLImageElement.
    /// Usage:
    ///   await renderer.createTexture(u8array)
    ///   await renderer.createTexture("/path/or/url.png")
    ///   await renderer.createTexture("#imgId")
    #[wasm_bindgen(js_name = "createTexture")]
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    pub async fn create_texture_js(
        &self,
        input: &JsValue,
    ) -> Result<crate::texture::Texture, JsError> {
        let input_converted: TextureInput = input.try_into()?;
        Ok(self.create_texture(input_converted).await?)
    }

    #[wasm_bindgen(js_name = "createTextureWithSize")]
    #[lsp_doc("docs/api/core/renderer/create_texture_with_size.md")]
    pub async fn create_texture_with_size_js(
        &self,
        input: &JsValue,
        size: &JsValue,
    ) -> Result<crate::texture::Texture, JsError> {
        let size: crate::Size = size.try_into()?;
        let input_converted: TextureInput = input.try_into()?;
        Ok(self.create_texture_with_size(input_converted, size).await?)
    }

    #[wasm_bindgen(js_name = "createTextureTarget")]
    #[lsp_doc("docs/api/core/renderer/create_texture_target.md")]
    pub async fn create_texture_target_js(
        &self,
        size: &JsValue,
    ) -> Result<crate::TextureTarget, JsError> {
        let size: Size = size.try_into()?;
        let target = self.create_texture_target(size).await?;
        Ok(target)
    }

    #[wasm_bindgen(js_name = "createTextureWithFormat")]
    #[lsp_doc("docs/api/core/renderer/create_texture_with_format.md")]
    pub async fn create_texture_with_format_js(
        &self,
        input: &JsValue,
        format: crate::TextureFormat,
    ) -> Result<crate::texture::Texture, JsError> {
        let input_converted: TextureInput = input.try_into()?;
        Ok(self
            .create_texture_with_format(input_converted, format)
            .await?)
    }

    #[wasm_bindgen(js_name = "createTextureWith")]
    #[lsp_doc("docs/api/core/renderer/create_texture_with.md")]
    pub async fn create_texture_with_js(
        &self,
        input: &JsValue,
        options: &JsValue,
    ) -> Result<crate::texture::Texture, JsError> {
        // Accept either a bare Size (arrays/typed arrays/object) or an object with fields.
        let input_converted: TextureInput = input.try_into()?;
        if let Ok(size) = Size::try_from(options) {
            let opts = crate::texture::TextureOptions {
                size: Some(size),
                ..Default::default()
            };
            return Ok(self.create_texture_with(input_converted, opts).await?);
        }
        // Fallback: try object with optional size/format (sampler optional; ignored here)
        use js_sys::Reflect;
        let mut opts = crate::texture::TextureOptions::default();
        if let Ok(v) = Reflect::get(options, &JsValue::from_str("size")) {
            if !v.is_undefined() && !v.is_null() {
                if let Ok(sz) = Size::try_from(&v) {
                    opts.size = Some(sz);
                }
            }
        }
        if let Ok(v) = Reflect::get(options, &JsValue::from_str("format")) {
            if let Some(n) = v.as_f64() {
                // wasm-bindgen enums are numeric in JS
                let code = n as u32;
                // Safe: TextureFormat has TryFrom<u32> via FromPrimitive in bindgen; fall back to default
                opts.format = match code {
                    0 => crate::TextureFormat::R8Unorm,
                    1 => crate::TextureFormat::Rg8Unorm,
                    2 => crate::TextureFormat::Rgba8Unorm,
                    3 => crate::TextureFormat::Rgba8UnormSrgb,
                    4 => crate::TextureFormat::Bgra8Unorm,
                    5 => crate::TextureFormat::Rgba16Unorm,
                    6 => crate::TextureFormat::Rgba32Float,
                    7 => crate::TextureFormat::Rgba32Uint,
                    8 => crate::TextureFormat::Rgba32Sint,
                    9 => crate::TextureFormat::Depth32Float,
                    10 => crate::TextureFormat::Rgba,
                    11 => crate::TextureFormat::Bgra,
                    12 => crate::TextureFormat::Lab,
                    13 => crate::TextureFormat::L8,
                    _ => crate::TextureFormat::default(),
                };
            }
        }
        Ok(self.create_texture_with(input_converted, opts).await?)
    }

    #[wasm_bindgen(js_name = "createStorageTexture")]
    #[lsp_doc("docs/api/core/renderer/create_storage_texture.md")]
    pub async fn create_storage_texture_js(
        &self,
        size: &JsValue,
        format: crate::TextureFormat,
        usage: Option<u32>,
    ) -> Result<crate::texture::Texture, JsError> {
        let size: Size = size.try_into()?;
        let usage_flags = usage.map(|bits| wgpu::TextureUsages::from_bits_truncate(bits));
        Ok(self
            .create_storage_texture(size, format, usage_flags)
            .await?)
    }

    #[wasm_bindgen(js_name = "createDepthTexture")]
    #[lsp_doc("docs/api/core/renderer/create_depth_texture.md")]
    pub async fn create_depth_texture_js(&self, size: &JsValue) -> Result<Texture, JsError> {
        let size: Size = size.try_into()?;
        Ok(self.create_depth_texture(size).await?)
    }

    #[wasm_bindgen(js_name = "render")]
    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render_js(&self, renderable: &JsValue, target: &JsValue) -> Result<(), RendererError> {
        // Canvas target
        if let Ok(canvas_target) = CanvasTarget::try_from(target) {
            if let Ok(shader) = Shader::try_from(renderable) {
                return self.render(&shader, &canvas_target);
            } else if let Ok(pass) = Pass::try_from(renderable) {
                return self.render(&pass, &canvas_target);
            } else if let Ok(frame) = Frame::try_from(renderable) {
                return self.render(&frame, &canvas_target);
            } else if let Ok(mesh) = Mesh::try_from(renderable) {
                return self.render(&mesh, &canvas_target);
            } else if Array::is_array(renderable) {
                for item in Array::from(renderable) {
                    self.render_js(&item, target)?;
                }
                return Ok(());
            }
        // Texture target
        } else if let Ok(texture_target) = TextureTarget::try_from(target) {
            if let Ok(shader) = Shader::try_from(renderable) {
                return self.render(&shader, &texture_target);
            } else if let Ok(pass) = Pass::try_from(renderable) {
                return self.render(&pass, &texture_target);
            } else if let Ok(frame) = Frame::try_from(renderable) {
                return self.render(&frame, &texture_target);
            } else if let Ok(mesh) = Mesh::try_from(renderable) {
                return self.render(&mesh, &texture_target);
            } else if Array::is_array(renderable) {
                for item in Array::from(renderable) {
                    self.render_js(&item, target)?;
                }
                return Ok(());
            }
        }

        Err(RendererError::Error(
            "Invalid target type in render".to_string(),
        ))
    }
}
