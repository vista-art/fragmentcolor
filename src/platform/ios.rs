//
// Copyright (C) Artizans - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited.
//

use std::sync::Arc;

use photogeometry::Rect;

use crate::{ffi, Bitmap, Destination, Image, PixelFormat};
use core_graphics::geometry::CGSize;
use objc::*;

const BACKENDS: wgpu::Backends = wgpu::Backends::METAL;

#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct Context {
    wrapped: Arc<crate::Context>,
}

async fn headless() -> crate::Context {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: BACKENDS,
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = ffi::platform::all::request_device(&adapter).await;

    crate::Context::new(device, queue)
}

#[cfg_attr(mobile, uniffi::export)]
impl Context {
    #[cfg_attr(mobile, uniffi::constructor)]
    pub async fn headless() -> Self {
        Context {
            wrapped: headless().await.into(),
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
pub struct Stage {
    surface: Option<wgpu::Surface<'static>>,
    wrapped: Arc<crate::Stage>,
}

#[cfg_attr(mobile, uniffi::export)]
impl Stage {
    #[cfg_attr(mobile, uniffi::constructor)]
    pub async fn headless() -> Self {
        let context = headless().await;
        Self {
            surface: None,
            wrapped: crate::Stage::new(context).into(),
        }
    }

    /// NOTE: Stage needs a raw pointer to connect with the CAMetalLayer.
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

        let (device, queue) = ffi::platform::all::request_device(&adapter).await;

        let capabilitiess = surface.get_capabilities(&adapter);
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilitiess.formats[0].remove_srgb_suffix(),
            width: u32::max(drawable_width, 1),
            height: u32::max(drawable_height, 1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilitiess.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let stage = crate::Stage::new(crate::Context::new(device, queue));
        surface.configure(stage.device(), &surface_configuration);

        Self {
            surface: Some(surface),
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

    pub async fn render_bitmap(
        &self,
        composition: &ffi::Composition,
        pixel_format: PixelFormat,
    ) -> Option<Arc<Bitmap>> {
        let composition = composition.wrapped.read().unwrap().clone();

        self.wrapped
            .render_bitmap(&composition, pixel_format)
            .await
            .ok()
            .map(|it| it.into())
    }
}
