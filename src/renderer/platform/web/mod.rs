use crate::{
    Mesh, Pass, Renderer, RendererError, Shader, Size, Texture, TextureData, TextureTarget,
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

    /// JS: Create a Texture from any input shape — Uint8Array bytes, URL
    /// string, file path, CSS selector, HTMLImageElement, ImageData, OffscreenCanvas,
    /// HTMLCanvasElement, or a `TextureMipChain` handle (built off-thread via
    /// `TextureMipChain.prepare`). Optional second argument is an options object
    /// `{ size?, format?, mipmaps?, sampler? }`. When `size` is present, `bytes`
    /// is treated as raw pixel data; otherwise it's decoded as an encoded image.
    #[wasm_bindgen(js_name = "createTexture")]
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    pub async fn create_texture_js(
        &self,
        input: &JsValue,
        options: Option<JsValue>,
    ) -> Result<crate::texture::Texture, JsError> {
        let data: TextureData = input.try_into()?;
        let opts = match options {
            None => crate::texture::TextureOptions::default(),
            Some(value) if value.is_undefined() || value.is_null() => {
                crate::texture::TextureOptions::default()
            }
            Some(value) => js_to_texture_options(&value)?,
        };
        let input = crate::texture::TextureInput {
            data,
            options: opts,
        };
        Ok(self.create_texture(input).await?)
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

    #[wasm_bindgen(js_name = "createStorageTexture")]
    #[lsp_doc("docs/api/core/renderer/create_storage_texture.md")]
    pub async fn create_storage_texture_js(
        &self,
        size: &JsValue,
        format: crate::TextureFormat,
        data: Option<js_sys::Uint8Array>,
        usage: Option<u32>,
    ) -> Result<crate::texture::Texture, JsError> {
        let size: Size = size.try_into()?;
        let data_vec = data.map(|arr| arr.to_vec());
        let input = crate::TextureInput {
            data: match data_vec {
                Some(bytes) => crate::TextureData::Bytes(bytes),
                None => crate::TextureData::Empty,
            },
            options: crate::TextureOptions {
                size: Some(size),
                format,
                usage,
                ..Default::default()
            },
        };
        Ok(self.create_storage_texture(input).await?)
    }

    #[wasm_bindgen(js_name = "createDepthTexture")]
    #[lsp_doc("docs/api/core/renderer/create_depth_texture.md")]
    pub async fn create_depth_texture_js(&self, size: &JsValue) -> Result<Texture, JsError> {
        let size: Size = size.try_into()?;
        Ok(self.create_depth_texture(size).await?)
    }

    #[wasm_bindgen(js_name = "unregisterTexture")]
    #[lsp_doc("docs/api/core/renderer/unregister_texture.md")]
    pub fn unregister_texture_js(&self, texture_id: &JsValue) -> Result<(), RendererError> {
        let id = crate::texture::js_to_texture_id(texture_id)?;
        self.unregister_texture(id)
    }

    /// No-op on WASM — the browser drives submission readiness. Provided for
    /// API parity; callers that need a sync point on the web should await a
    /// readback (`Renderer.readTexture` / `Texture.getImage`).
    #[wasm_bindgen(js_name = "waitIdle")]
    #[lsp_doc("docs/api/core/renderer/hidden/wait_js.md")]
    pub fn wait_js(&self) -> Result<(), RendererError> {
        self.wait()
    }

    #[wasm_bindgen(js_name = "readTexture")]
    #[lsp_doc("docs/api/core/renderer/read_texture.md")]
    pub async fn read_texture_js(
        &self,
        texture_id: &JsValue,
    ) -> Result<js_sys::Uint8Array, JsError> {
        let id = crate::texture::js_to_texture_id(texture_id)?;
        let bytes = self
            .read_texture(id)
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(js_sys::Uint8Array::from(bytes.as_slice()))
    }

    #[wasm_bindgen(js_name = "createExternalTexture")]
    #[lsp_doc("docs/api/core/renderer/hidden/create_external_texture_js.md")]
    pub fn create_external_texture_js(
        &self,
        video: &web_sys::HtmlVideoElement,
    ) -> Result<crate::renderer::external_texture::ExternalTextureHandle, RendererError> {
        self.create_external_texture(video)
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

/// Decode a JS value into [`crate::texture::TextureOptions`].
///
/// Accepts any of:
/// - A bare Size (`[w, h]`, `{width, height}`, etc.) — sets `options.size`,
///   leaving the rest at defaults. Equivalent to the old
///   `createTextureWithSize(input, size)` shortcut.
/// - A bare `TextureFormat` (numeric enum) — sets `options.format`. Equivalent
///   to the old `createTextureWithFormat(input, format)` shortcut.
/// - An options object `{ size?, format?, mipmaps? }` — explicit field-by-field.
fn js_to_texture_options(value: &JsValue) -> Result<crate::texture::TextureOptions, JsError> {
    // Bare Size shorthand (matches the old createTextureWithSize ergonomics).
    if let Ok(size) = Size::try_from(value) {
        return Ok(crate::texture::TextureOptions {
            size: Some(size),
            ..Default::default()
        });
    }
    // Bare TextureFormat shorthand (matches the old createTextureWithFormat).
    if let Some(n) = value.as_f64() {
        return Ok(crate::texture::TextureOptions {
            format: js_format_from_code(n as u32),
            ..Default::default()
        });
    }
    // Object with optional fields.
    use js_sys::Reflect;
    let mut opts = crate::texture::TextureOptions::default();
    if let Ok(v) = Reflect::get(value, &JsValue::from_str("size"))
        && !v.is_undefined()
        && !v.is_null()
        && let Ok(sz) = Size::try_from(&v)
    {
        opts.size = Some(sz);
    }
    if let Ok(v) = Reflect::get(value, &JsValue::from_str("format"))
        && let Some(n) = v.as_f64()
    {
        opts.format = js_format_from_code(n as u32);
    }
    if let Ok(v) = Reflect::get(value, &JsValue::from_str("mipmaps"))
        && let Some(b) = v.as_bool()
    {
        opts.mipmaps = b;
    }
    Ok(opts)
}

/// Map the wasm-bindgen numeric enum code back to a `TextureFormat`. Mirrors
/// the variant order in `src/texture/format.rs` — keep in sync.
fn js_format_from_code(code: u32) -> crate::TextureFormat {
    match code {
        0 => crate::TextureFormat::R8Unorm,
        1 => crate::TextureFormat::Rg8Unorm,
        2 => crate::TextureFormat::R16Unorm,
        3 => crate::TextureFormat::Rg16Unorm,
        4 => crate::TextureFormat::Rgba8Unorm,
        5 => crate::TextureFormat::Rgba8UnormSrgb,
        6 => crate::TextureFormat::Bgra8Unorm,
        7 => crate::TextureFormat::Rgba16Unorm,
        8 => crate::TextureFormat::Rgba16Float,
        9 => crate::TextureFormat::Rgba32Float,
        10 => crate::TextureFormat::Rgba32Uint,
        11 => crate::TextureFormat::Rgba32Sint,
        12 => crate::TextureFormat::Depth32Float,
        13 => crate::TextureFormat::Rgba,
        14 => crate::TextureFormat::Bgra,
        15 => crate::TextureFormat::Lab,
        16 => crate::TextureFormat::L8,
        _ => crate::TextureFormat::default(),
    }
}
