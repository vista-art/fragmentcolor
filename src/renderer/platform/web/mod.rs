use crate::bindings::web::{describe, read_text, read_unsigned};
use crate::error::{ErrorClass, WebError, defer};
use crate::texture::{js_to_texture_bytes, read_format, read_texture_input, read_usage};
use crate::{
    LogLevel, Mesh, Pass, PassObject, Renderable, Renderer, Scene, Shader, Size, Texture,
    TextureData, TextureTarget, target::CanvasTarget,
};
use js_sys::Array;
use lsp_doc::lsp_doc;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// A canvas a window target draws into.
pub(crate) enum Canvas {
    Html(web_sys::HtmlCanvasElement),
    Offscreen(web_sys::OffscreenCanvas),
}

impl Canvas {
    pub(crate) fn size(&self) -> wgpu::Extent3d {
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

    #[wasm_bindgen(js_name = "setLogLevel")]
    #[lsp_doc("docs/api/core/renderer/set_log_level.md")]
    pub fn set_log_level_js(
        #[wasm_bindgen(unchecked_param_type = "LogLevel")] level: &JsValue,
    ) -> Result<(), WebError> {
        Renderer::set_log_level(read_log_level(level)?);
        Ok(())
    }

    #[wasm_bindgen(js_name = "createTarget")]
    #[lsp_doc("docs/api/core/renderer/create_target.md")]
    pub async fn create_target_js(&self, canvas: JsValue) -> Result<CanvasTarget, WebError> {
        let canvas = if let Some(canvas) = canvas.dyn_ref::<web_sys::HtmlCanvasElement>() {
            Canvas::Html(canvas.clone())
        } else if let Ok(canvas) = canvas.dyn_into::<web_sys::OffscreenCanvas>() {
            Canvas::Offscreen(canvas)
        } else {
            return Err(WebError::type_error("Failed to convert input to Canvas"));
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

    /// Create a Texture from bytes, a URL, a CSS selector, an image, an
    /// `ImageData`, a canvas, a `Mipmap` or another `Texture`. The optional
    /// second argument is a size, a `TextureFormat`, or an options object
    /// `{ size?, format?, mipmaps?, sampler?, usage? }`. With a size, bytes
    /// are raw pixels; without one they are decoded as an encoded image.
    #[wasm_bindgen(js_name = "createTexture")]
    #[lsp_doc("docs/api/core/renderer/create_texture.md")]
    pub async fn create_texture_js(
        &self,
        input: &JsValue,
        options: Option<JsValue>,
    ) -> Result<Texture, WebError> {
        let input = read_texture_input(input, options.as_ref(), "Renderer.createTexture")?;
        Ok(self.create_texture(input).await?)
    }

    #[wasm_bindgen(js_name = "createTextureTarget")]
    #[lsp_doc("docs/api/core/renderer/create_texture_target.md")]
    pub async fn create_texture_target_js(
        &self,
        size: &JsValue,
    ) -> Result<crate::TextureTarget, WebError> {
        let size: Size = size.try_into()?;
        let target = self.create_texture_target(size).await?;
        Ok(target)
    }

    #[wasm_bindgen(js_name = "createStorageTexture")]
    #[lsp_doc("docs/api/core/renderer/create_storage_texture.md")]
    pub async fn create_storage_texture_js(
        &self,
        size: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "TextureFormat")] format: &JsValue,
        #[wasm_bindgen(
            unchecked_optional_param_type = "Uint8Array | ArrayBuffer | ArrayLike<number> | ImageData | HTMLCanvasElement | OffscreenCanvas | null"
        )]
        data: Option<JsValue>,
        #[wasm_bindgen(unchecked_optional_param_type = "TextureUsage | number")] usage: Option<
            JsValue,
        >,
    ) -> Result<crate::texture::Texture, WebError> {
        let given = |value: &JsValue| !value.is_undefined() && !value.is_null();
        let size: Size = size.try_into()?;
        let format = read_format(format)?;
        let usage = usage.filter(given).as_ref().map(read_usage).transpose()?;
        let data = match data.filter(given) {
            Some(bytes) => {
                let label = "Renderer.createStorageTexture: data";
                let (bytes, _) = js_to_texture_bytes(&bytes, Some(size), format.to_wgpu(), label)?;
                TextureData::Bytes(bytes)
            }
            None => TextureData::Empty,
        };
        let options = crate::TextureOptions {
            size: Some(size),
            format,
            usage,
            ..Default::default()
        };
        let input = crate::TextureInput { data, options };
        Ok(self.create_storage_texture(input).await?)
    }

    #[wasm_bindgen(js_name = "createDepthTexture")]
    #[lsp_doc("docs/api/core/renderer/create_depth_texture.md")]
    pub async fn create_depth_texture_js(&self, size: &JsValue) -> Result<Texture, WebError> {
        let size: Size = size.try_into()?;
        Ok(self.create_depth_texture(size).await?)
    }

    #[wasm_bindgen(js_name = "unregisterTexture")]
    #[lsp_doc("docs/api/core/renderer/unregister_texture.md")]
    pub fn unregister_texture_js(&self, texture_id: &JsValue) -> Result<(), WebError> {
        let id = crate::texture::js_to_texture_id(texture_id)?;
        Ok(self.unregister_texture(id)?)
    }

    // The readbacks and load are plain functions, so each call takes its start
    // before it returns the promise, whose future first runs on a later microtask.
    #[wasm_bindgen(js_name = "readTexture", unchecked_return_type = "Promise<Uint8Array>")]
    #[lsp_doc("docs/api/core/renderer/read_texture.md")]
    pub fn read_texture_js(&self, texture_id: &JsValue) -> js_sys::Promise {
        let start = self.start();
        let renderer = self.share();
        let id = crate::texture::js_to_texture_id(texture_id);
        defer(async move {
            let id = id?;
            let bytes = renderer.read_texture_from(id, start).await?;
            Ok(js_sys::Uint8Array::from(bytes.as_slice()).into())
        })
    }

    #[wasm_bindgen(js_name = "load", unchecked_return_type = "Promise<void>")]
    #[lsp_doc("docs/api/core/renderer/load.md")]
    pub fn load_js(
        &self,
        #[wasm_bindgen(
            unchecked_param_type = "Shader | Pass | Mesh | Scene | Array<Shader | Pass | Mesh | Scene>"
        )]
        renderable: &JsValue,
    ) -> js_sys::Promise {
        let start = self.start();
        let renderer = self.share();
        let renderables = RenderableList::read(renderable, "Renderer.load");
        defer(async move {
            let renderables = renderables?;
            renderer.load_from(&renderables, start).await?;
            Ok(JsValue::UNDEFINED)
        })
    }

    #[wasm_bindgen(js_name = "readStorage", unchecked_return_type = "Promise<Uint8Array>")]
    #[lsp_doc("docs/api/core/renderer/read_storage.md")]
    pub fn read_storage_js(
        &self,
        shader: &Shader,
        #[wasm_bindgen(unchecked_param_type = "string")] binding: &JsValue,
    ) -> js_sys::Promise {
        let start = self.start();
        let renderer = self.share();
        let shader = shader.clone();
        let binding = read_text(binding, "Renderer.readStorage: binding");
        defer(async move {
            let binding = binding?;
            let bytes = renderer.read_storage_from(&shader, &binding, start).await?;
            Ok(js_sys::Uint8Array::from(bytes.as_slice()).into())
        })
    }

    #[wasm_bindgen(js_name = "render")]
    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render_js(
        &self,
        #[wasm_bindgen(
            unchecked_param_type = "Shader | Pass | Mesh | Scene | Array<Shader | Pass | Mesh | Scene>"
        )]
        renderable: &JsValue,
        #[wasm_bindgen(unchecked_param_type = "WindowTarget | TextureTarget")] target: &JsValue,
    ) -> Result<(), WebError> {
        let renderables = RenderableList::read(renderable, "Renderer.render")?;
        if let Ok(window) = CanvasTarget::try_from(target) {
            return Ok(self.render(&renderables, &window)?);
        }
        if let Ok(texture) = TextureTarget::try_from(target) {
            return Ok(self.render(&renderables, &texture)?);
        }
        Err(WebError::type_error(
            "Renderer.render: expected a WindowTarget or a TextureTarget",
        ))
    }
}

/// A `LogLevel` from JS, a number in variant order, or the error that names the value.
fn read_log_level(value: &JsValue) -> Result<LogLevel, WebError> {
    let code = value.as_f64().ok_or_else(|| {
        WebError::type_error(&format!(
            "Renderer.setLogLevel: level must be a LogLevel, got {}",
            describe(value)
        ))
    })?;
    match read_unsigned(code, "Renderer.setLogLevel: level")? {
        0 => Ok(LogLevel::Off),
        1 => Ok(LogLevel::Error),
        2 => Ok(LogLevel::Warn),
        3 => Ok(LogLevel::Info),
        4 => Ok(LogLevel::Debug),
        5 => Ok(LogLevel::Trace),
        _ => Err(WebError::new(
            ErrorClass::InvalidInput,
            &format!("Renderer.setLogLevel: {code} is not a LogLevel"),
        )),
    }
}

/// What `render`, `load` and `Pass.require` take on the web: a `Shader`, `Pass`,
/// `Mesh` or `Scene`, or an array of them. An array renders as one frame, each
/// item's passes in order, and as a dependency it links each item's roots.
pub(crate) struct RenderableList(Vec<Box<dyn Renderable>>);

impl RenderableList {
    /// Reads a renderable or an array of them. Errors start with `method`.
    pub(crate) fn read(value: &JsValue, method: &str) -> Result<Self, WebError> {
        if let Some(item) = read_renderable(value) {
            return Ok(Self(vec![item]));
        }
        let Some(array) = value.dyn_ref::<Array>() else {
            return Err(WebError::type_error(&format!(
                "{method}: expected a Shader, Pass, Mesh, Scene, or an array of them"
            )));
        };
        let read = |(index, item): (usize, JsValue)| {
            read_renderable(&item).ok_or_else(|| {
                WebError::type_error(&format!(
                    "{method}: item {index} is not a Shader, Pass, Mesh, or Scene"
                ))
            })
        };
        array
            .iter()
            .enumerate()
            .map(read)
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

impl Renderable for RenderableList {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        self.0
            .iter()
            .flat_map(|item| item.passes().to_vec())
            .collect()
    }

    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        self.0
            .iter()
            .flat_map(|item| item.roots().to_vec())
            .collect()
    }

    fn peek_passes(&self) -> Arc<[Arc<PassObject>]> {
        self.0
            .iter()
            .flat_map(|item| item.peek_passes().to_vec())
            .collect()
    }
}

fn read_renderable(value: &JsValue) -> Option<Box<dyn Renderable>> {
    if let Ok(shader) = Shader::try_from(value) {
        return Some(Box::new(shader));
    }
    if let Ok(pass) = Pass::try_from(value) {
        return Some(Box::new(pass));
    }
    if let Ok(mesh) = Mesh::try_from(value) {
        return Some(Box::new(mesh));
    }
    if let Ok(scene) = Scene::try_from(value) {
        return Some(Box::new(scene));
    }
    None
}
