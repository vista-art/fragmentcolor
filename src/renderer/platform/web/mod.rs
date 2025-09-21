use crate::{Frame, Pass, Renderer, RendererError, Shader, Size};
use js_sys::Array;
use lsp_doc::lsp_doc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
pub mod target;
pub use target::*;

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
    /// Creates a new Renderer
    pub fn new_js() -> Self {
        Self::new()
    }

    #[wasm_bindgen(js_name = "createTarget")]
    #[lsp_doc("docs/api/core/renderer/create_target.md")]
    pub async fn create_target_js(&self, canvas: JsValue) -> Result<CanvasTarget, JsError> {
        let canvas = if canvas.has_type::<web_sys::HtmlCanvasElement>() {
            let canvas = canvas
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .map_err(|_| JsError::new("Failed to convert input to HtmlCanvasElement"))?;
            Canvas::Html(canvas)
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
        input: JsValue,
    ) -> Result<crate::texture::Texture, JsError> {
        use js_sys::Uint8Array;
        use web_sys::HtmlImageElement;

        // Case 1: Uint8Array
        if let Some(u8a) = input.dyn_ref::<Uint8Array>() {
            let mut bytes = vec![0u8; u8a.length() as usize];
            u8a.copy_to(&mut bytes[..]);
            return self
                .create_texture(crate::texture::TextureInput::Bytes(bytes))
                .await
                .map_err(|e| JsError::new(&format!("{e}")));
        }

        // Case 2: String: URL or selector
        if let Some(s) = input.as_string() {
            let url = if s.starts_with('#') || s.starts_with('.') {
                // Treat as selector
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    if let Ok(Some(elem)) = doc.query_selector(&s) {
                        if let Ok(img) = elem.dyn_into::<HtmlImageElement>() {
                            img.src()
                        } else {
                            s.clone()
                        }
                    } else {
                        s.clone()
                    }
                } else {
                    s.clone()
                }
            } else {
                s.clone()
            };
            let bytes = crate::net::fetch_bytes(&url)
                .await
                .map_err(|_| JsError::new("fetch bytes error for URL"))?;
            return self
                .create_texture(crate::texture::TextureInput::Bytes(bytes))
                .await
                .map_err(|e| JsError::new(&format!("{e}")));
        }

        // Case 3: HTMLImageElement
        if input.has_type::<HtmlImageElement>() {
            let img: HtmlImageElement =
                input.dyn_into().map_err(|_| JsError::new("Not an image"))?;
            let bytes = crate::net::fetch_bytes(&img.src())
                .await
                .map_err(|_| JsError::new("fetch bytes error for HtmlImageElement"))?;
            return self
                .create_texture(crate::texture::TextureInput::Bytes(bytes))
                .await
                .map_err(|e| JsError::new(&format!("{e}")));
        }

        Err(JsError::new("Unsupported input for createTexture"))
    }

    #[wasm_bindgen(js_name = "createTextureTarget")]
    #[lsp_doc("docs/api/core/renderer/create_texture_target.md")]
    pub async fn create_texture_target_js(&self, size: &JsValue) -> Result<TextureTarget, JsError> {
        let size: Size = size
            .try_into()
            .map_err(|e: crate::size::error::SizeError| JsError::new(&format!("{e}")))?;

        let target = self
            .create_texture_target(size)
            .await
            .map_err(|e| JsError::new(&format!("{e}")))?;

        Ok(TextureTarget::from(target))
    }

    #[wasm_bindgen(js_name = "render")]
    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render_js(&self, renderable: &JsValue, target: &JsValue) -> Result<(), RendererError> {
        //
        // Canvas target
        if let Ok(canvas_target) = CanvasTarget::try_from(target) {
            if let Ok(shader) = Shader::try_from(renderable) {
                return self.render(&shader, &canvas_target);
            } else if let Ok(pass) = Pass::try_from(renderable) {
                return self.render(&pass, &canvas_target);
            } else if let Ok(frame) = Frame::try_from(renderable) {
                return self.render(&frame, &canvas_target);
            } else if Array::is_array(renderable) {
                for item in Array::from(renderable) {
                    self.render_js(&item, target)?;
                }
                return Ok(());
            }
        //
        // Texture target
        } else if let Ok(texture_target) = TextureTarget::try_from(target) {
            if let Ok(shader) = Shader::try_from(renderable) {
                return self.render(&shader, &texture_target);
            } else if let Ok(pass) = Pass::try_from(renderable) {
                return self.render(&pass, &texture_target);
            } else if let Ok(frame) = Frame::try_from(renderable) {
                return self.render(&frame, &texture_target);
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
