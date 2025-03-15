use std::sync::Arc;

use photogeometry::Rect;

use crate::{ffi, Bitmap, Destination, Image, PixelFormat};
use core_graphics::geometry::CGSize;
use objc::*;

const BACKENDS: wgpu::Backends = wgpu::Backends::METAL;

#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct Renderer {
    wrapped: Arc<crate::Renderer>,
}

#[cfg_attr(mobile, uniffi::export)]
impl Renderer {
    #[cfg_attr(mobile, uniffi::constructor)]
    pub async fn headless() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKENDS,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = crate::platform::all::request_device(&adapter).await;

        let renderer = crate::Renderer::new(device, queue).await;

        Renderer {
            wrapped: renderer.into(),
        }
    }

    pub async fn render_bitmap(
        &self,
        image: &Image,
        bounds: Option<Arc<Rect>>,
        pixel_format: PixelFormat,
    ) -> Option<Arc<Bitmap>> {
        self.wrapped
            .render_bitmap(image, bounds.map(|it| *it.as_ref()), pixel_format)
            .await
            .ok()
            .map(|it| it.into())
    }
}

#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct FragmentColor {
    surface: Option<wgpu::Surface<'static>>,
    wrapped: Arc<crate::FragmentColor>,
}

#[cfg_attr(mobile, uniffi::export)]
impl FragmentColor {
    #[cfg_attr(mobile, uniffi::constructor)]
    pub async fn headless() -> Self {
        let context = headless().await;
        Self {
            surface: None,
            wrapped: crate::FragmentColor::new(context).into(),
        }
    }

    /// NOTE: FragmentColor needs a raw pointer to connect with the CAMetalLayer.
    /// Unfortunately uniffi currently does not support interfacing with raw
    /// pointers.
    ///
    /// As of April 2024, early discussions are happening to add support to it:
    /// https://github.com/mozilla/uniffi-rs/issues/1946
    ///
    /// We can remove this ugly hack once uniffi supports raw pointers
    #[cfg_attr(mobile, uniffi::constructor)]
    pub async fn in_metal_layer(metal_layer_ptr: u64) -> Self {
        let metal_layer = metal_layer_ptr as *mut objc::runtime::Object;
        let (drawable_width, drawable_height) = unsafe {
            let size: CGSize = objc::msg_send![metal_layer, drawableSize];
            (size.width as u32, size.height as u32)
        };

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKENDS,
            ..Default::default()
        });

        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(
                    metal_layer as _,
                ))
                .expect("Failed to create surface")
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = crate::platform::all::request_device(&adapter).await;
        let config = crate::platform::all::configure_surface(&device, &adapter, &surface, &size);
        let renderer = crate::FragmentColor::new(crate::Renderer::new(device, queue));

        Self {
            renderer: Some(surface),
            wrapped: stage.into(),
        }
    }

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
}
