use crate::{CanvasTarget, FragmentColor, InitializationError, Shader, ShaderError};
use std::sync::Arc;
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
pub struct RendererTargetWrapper {
    renderer: Arc<Renderer>,
    target: Arc<CanvasTarget>,
}

#[wasm_bindgen]
pub struct Renderer {
    inner: crate::renderer::Renderer,
}

#[wasm_bindgen]
impl FragmentColor {
    pub async fn init(canvas: JsValue) -> Result<Renderer, JsError> {
        let canvas = if canvas.has_type::<web_sys::HtmlCanvasElement>() {
            let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
            Canvas::Html(canvas)
        } else if let Ok(canvas) = canvas.dyn_into::<web_sys::OffscreenCanvas>() {
            Canvas::Offscreen(canvas)
        } else {
            return Err(JsError::new("Failed to convert input to OffscreenCanvas"));
        };

        Ok(FragmentColor::init_renderer_and_target(canvas).await?)
    }

    async fn init_renderer_and_target(canvas: Canvas) -> Result<Renderer, InitializationError> {
        let instance = wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        })
        .await;

        let size = canvas.size();
        let surface = match canvas {
            Canvas::Html(canvas) => instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas))?,
            Canvas::Offscreen(canvas) => {
                instance.create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas))?
            }
        };

        let adapter = crate::platform::all::request_adapter(&instance, Some(&surface)).await?;
        let (device, queue) = crate::platform::all::request_device(&adapter).await?;
        let config = crate::platform::all::configure_surface(&device, &adapter, &surface, &size);

        let target = CanvasTarget::new(surface, config);
        let renderer = Renderer::init(device, queue);

        Ok(RendererTargetWrapper { renderer, target })
    }
}

#[wasm_bindgen]
impl Renderer {
    /// Creates a headless renderer by default
    pub async fn new() -> Result<Renderer, JsError> {
        Renderer::headless().await
    }

    /// Creates a headless Renderer
    pub async fn headless() -> Result<Renderer, JsError> {
        let instance = wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        })
        .await;

        let backends = wgpu::Instance::enabled_backend_features();

        let adapter = if !backends.contains(wgpu::Backends::BROWSER_WEBGPU) {
            // Create a DOM canvas element.
            // This is needed to make adapter creation work in WebGL.
            //
            // We must create the surface from the same Instance we create the adapter,
            // and the surface must remain alive during the call to request_adapter(),
            // even though it can be immediately dropped afterwards.
            //
            // Relevant discussion: https://github.com/gfx-rs/wgpu/issues/5190
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas))?;

            crate::platform::all::request_adapter(&instance, Some(&surface)).await?
        } else {
            crate::platform::all::request_headless_adapter(&instance).await?
        };

        let (device, queue) = crate::platform::all::request_device(&adapter).await?;

        Ok(Renderer::init(device, queue))
    }
}

impl Shader {
    pub async fn fetch(url: &str) -> Result<Self, ShaderError> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::{JsFuture, future_to_promise};
        use web_sys::Request;
        use web_sys::RequestInit;
        use web_sys::RequestMode;
        use web_sys::Response;

        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(url, &opts).expect("failed to create request");
        let window = web_sys::window().expect("no global `window` exists");
        let resp_promise = window.fetch_with_request(&request);
        let resp_value = future_to_promise(JsFuture::from(resp_promise));

        let resp: Response = resp_value.dyn_into().expect("not a Response");

        let jsvalue = JsFuture::from(resp.text().expect("failed to read response"))
            .await
            .expect("failed to read response");

        let body = jsvalue.as_string().expect("response not a string");

        Self::new(&body)
    }
}
