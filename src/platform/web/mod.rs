use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::{ffi, Bitmap, Destination, Image, PixelFormat};
use photogeometry::Rect;

const BACKENDS: wgpu::Backends = { wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU };

pub enum Canvas {
    Html(web_sys::HtmlCanvasElement),
    Offscreen(web_sys::OffscreenCanvas),
}

impl Renderer {
    #[wasm_bindgen]
    pub async fn headless() -> Self {
        let instance = wgpu::utils::new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor {
            backends: BACKENDS,
            ..Default::default()
        });

        // @TODO do backends contain BROWSER_WEBGPU?
        // if !instance.supports_any_backend(BACKENDS) {
        //      / create foe GL here with the canvas and surface
        // } else {
        //      / no need to create a canvas and surface
        // }}

        // Create a DOM canvas element
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>();

        // Needed to make adapter creation work in WebGL.
        // We must create_surface() from the same Instance we create the adapter,
        // and the surface must remain alive during the call to request_adapter(),
        // even though it can be immediately dropped afterwards.
        // Relevant discussion: https://github.com/gfx-rs/wgpu/issues/5190
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.unwrap()))
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = crate::platform::all::request_device(&adapter).await;

        device.on_uncaptured_error(Box::new(|error| {
            web_sys::console::error_1(&format!("Error: {:?}", error).into());
        }));

        Renderer {
            wrapped: crate::Renderer::init(device, queue),
        }
    }
}

pub struct FragmentColor {
    surface: Option<wgpu::Surface<'static>>,
    wrapped: crate::FragmentColor,
}

impl FragmentColor {
    #[wasm_bindgen]
    pub async fn init(canvas: Canvas) -> Self {
        let width = canvas.width();
        let height = canvas.height();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: BACKENDS,
            ..Default::default()
        });

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = crate::platform::all::request_device(&adapter).await;

        device.on_uncaptured_error(Box::new(|error| {
            web_sys::console::error_1(&format!("Error: {:?}", error).into());
        }));

        let capabilitiess = surface.get_capabilities(&adapter);
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilitiess.formats[0].remove_srgb_suffix(),
            width: u32::max(width, 1),
            height: u32::max(height, 1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilitiess.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let stage = crate::FragmentColor::new(crate::Renderer::new(device, queue));
        surface.configure(stage.device(), &surface_configuration);

        Self {
            surface: Some(surface),
            wrapped: stage,
        }
    }
}
