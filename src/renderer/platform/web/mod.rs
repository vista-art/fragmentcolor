use crate::Renderer;
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
    /// Creates a new Renderer
    pub fn new_js() -> Self {
        Self::new()
    }

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

    // @TODO
    // /// Creates a headless Renderer
    // pub async fn headless() -> Result<Renderer, JsError> {
    //     let instance = wgpu::util::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor {
    //         backends: wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU,
    //         ..Default::default()
    //     })
    //     .await;

    //     let backends = wgpu::Instance::enabled_backend_features();

    //     let adapter = if !backends.contains(wgpu::Backends::BROWSER_WEBGPU) {
    //         // Create a DOM canvas element.
    //         // This is needed to make adapter creation work in WebGL.
    //         //
    //         // We must create the surface from the same Instance we create the adapter,
    //         // and the surface must remain alive during the call to request_adapter(),
    //         // even though it can be immediately dropped afterwards.
    //         //
    //         // Relevant discussion: https://github.com/gfx-rs/wgpu/issues/5190
    //         let canvas = web_sys::window()
    //             .unwrap()
    //             .document()
    //             .unwrap()
    //             .create_element("canvas")
    //             .unwrap()
    //             .dyn_into::<web_sys::HtmlCanvasElement>()
    //             .unwrap();

    //         let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas))?;

    //         crate::platform::all::request_adapter(&instance, Some(&surface)).await?
    //     } else {
    //         crate::platform::all::request_headless_adapter(&instance).await?
    //     };

    //     let (device, queue) = crate::platform::all::request_device(&adapter).await?;

    //     Ok(Renderer::init(device, queue))
    // }
}
