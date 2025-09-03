use crate::{Frame, Pass, Renderer, Shader, ShaderError, Size};
use lsp_doc::lsp_doc;
use wasm_bindgen::JsCast;
use wasm_bindgen::convert::TryFromJsValue;
use std::convert::TryInto;
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
    #[lsp_doc("docs/api/renderer/constructor.md")]
    /// Creates a new Renderer
    pub fn new_js() -> Self {
        Self::new()
    }

    #[wasm_bindgen(js_name = "createTarget")]
    #[lsp_doc("docs/api/renderer/create_target.md")]
    pub async fn create_target(&self, canvas: JsValue) -> Result<CanvasTarget, JsError> {
        let canvas = if canvas.has_type::<web_sys::HtmlCanvasElement>() {
            let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
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

#[wasm_bindgen(js_name = "createTextureTarget")]
#[lsp_doc("docs/api/renderer/create_texture_target.md")]
pub async fn create_texture_target_js(&self, size: JsValue) -> Result<TextureTarget, JsError> {
    // Accept either a JS array (e.g., [w, h] or [w, h, d]), a typed array, a plain object
    // with width/height[/depth], or an exported Size instance
    let size: Size = size
        .try_into()
        .map_err(|e: crate::error::ShaderError| JsError::new(&format!("{e}")))?;

    let target = self
        .create_texture_target(size)
        .await
        .map_err(|e| JsError::new(&format!("{e}")))?;

    Ok(TextureTarget::from(target))
}

    #[wasm_bindgen(js_name = "render")]
    #[lsp_doc("docs/api/renderer/render.md")]
    pub fn render_js(&self, renderable: JsValue, target: JsValue) -> Result<(), ShaderError> {
        if let Ok(canvas_target) = CanvasTarget::try_from_js_value(target.clone()) {
            if let Ok(shader) = Shader::try_from_js_value(renderable.clone()) {
                return self.render(&shader, &canvas_target);
            } else if let Ok(pass) = Pass::try_from_js_value(renderable.clone()) {
                return self.render(&pass, &canvas_target);
            } else if let Ok(frame) = Frame::try_from_js_value(renderable) {
                return self.render(&frame, &canvas_target);
            } else {
                return Err(ShaderError::WasmError(
                    "Invalid object type in render".to_string(),
                ));
            };
        } else if let Ok(texture_target) = TextureTarget::try_from_js_value(target) {
            if let Ok(shader) = Shader::try_from_js_value(renderable.clone()) {
                return self.render(&shader, &texture_target);
            } else if let Ok(pass) = Pass::try_from_js_value(renderable.clone()) {
                return self.render(&pass, &texture_target);
            } else if let Ok(frame) = Frame::try_from_js_value(renderable) {
                return self.render(&frame, &texture_target);
            } else {
                return Err(ShaderError::WasmError(
                    "Invalid object type in render".to_string(),
                ));
            };
        } else {
            return Err(ShaderError::WasmError(
                "Invalid target type in render".to_string(),
            ));
        }
    }
}
