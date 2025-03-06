use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::{ffi, Bitmap, Destination, Image, PixelFormat};
use photogeometry::Rect;

// Ideally, we should use the default instance or ::all()
// and have WebGPU detected at runtime.
//
// This can be done when this upstream issue is fixed:
// https://github.com/gfx-rs/wgpu/issues/5332
//
// For now, listing anything other than "GL" will panic
// in WebGL context, even if the other backend is not used.
//
// For example, this will panic in Firefox
// let backends = { wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU };
//
// One workaround is to compile two WASM binaries, one for
// WebGPU and another for WebGL, and choose them from JS.

const BACKENDS: wgpu::Backends = wgpu::Backends::GL;

pub struct Renderer {
    wrapped: crate::Renderer,
}

impl Renderer {
    #[wasm_bindgen(js_name = headless)]
    pub async fn headless() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKENDS,
            ..Default::default()
        });

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

        let (device, queue) = ffi::platform::all::request_device(&adapter).await;

        device.on_uncaptured_error(Box::new(|error| {
            web_sys::console::error_1(&format!("Error: {:?}", error).into());
        }));

        Renderer {
            wrapped: crate::Renderer::new(device, queue),
        }
    }

    #[wasm_bindgen(js_name = renderBitmap)]
    pub async fn render_bitmap(
        &self,
        image: &Image,
        bounds: Option<Rect>,
        pixel_format: PixelFormat,
    ) -> Option<Bitmap> {
        // ImageData has no notion of padding or bpr, so we might as well remove
        // it here systematically.
        self.wrapped
            .render_bitmap(image, bounds, pixel_format)
            .await
            .ok()
            .map(|it| it.removing_padding())
    }
}

pub struct Stage {
    surface: Option<wgpu::Surface<'static>>,
    wrapped: crate::Stage,
}

impl Stage {
    #[wasm_bindgen(js_name = inCanvas)]
    pub async fn in_canvas(canvas: web_sys::HtmlCanvasElement) -> Self {
        let width = canvas.width();
        let height = canvas.height();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
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

        let (device, queue) = ffi::platform::all::request_device(&adapter).await;

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

        let stage = crate::Stage::new(crate::Renderer::new(device, queue));
        surface.configure(stage.device(), &surface_configuration);

        Self {
            surface: Some(surface),
            wrapped: stage,
        }
    }

    #[wasm_bindgen(js_name = headless)]
    pub async fn headless() -> Self {
        let context = Renderer::headless().await;

        Self {
            surface: None,
            wrapped: crate::Stage::new(context.wrapped),
        }
    }

    #[wasm_bindgen(js_name = draw)]
    pub fn draw(&self, composition: &ffi::Composition) {
        let Some(surface) = self.surface.as_ref() else {
            panic!("Cannot draw on a headless stage, use `render_bitmap` instead");
        };

        let composition = composition.wrapped.read().unwrap();

        let surface_texture = surface
            .get_current_texture()
            .expect("Failed to get texture");

        self.wrapped
            .render(&composition, Destination::Texture(&surface_texture.texture))
            .expect("Failed rendering");

        surface_texture.present();
    }

    #[wasm_bindgen(js_name = renderBitmap)]
    pub async fn render_bitmap(
        &self,
        composition: &ffi::Composition,
        pixel_format: PixelFormat,
    ) -> Option<Bitmap> {
        let composition = composition.wrapped.read().unwrap();

        // ImageData has no notion of padding or bpr, so we might as well remove
        // it here systematically.
        self.wrapped
            .render_bitmap(&composition, pixel_format)
            .await
            .ok()
            .map(|it| it.removing_padding())
    }
}
