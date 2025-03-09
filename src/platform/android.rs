use std::sync::Arc;

use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};

use jni::{objects::JClass, sys::jobject, JNIEnv};
use jni_fn::jni_fn;

use crate::Region;

use crate::{Bitmap, Destination, Image, PixelFormat};

const BACKENDS: wgpu::Backends = wgpu::Backends::VULKAN;

/// An implementation of HasWindowHandle + HasDisplayHandle for Android.
#[derive(Debug)]
struct AndroidNativeWindow {
    android_window: *mut ndk_sys::ANativeWindow,
}

unsafe impl Send for AndroidNativeWindow {}
unsafe impl Sync for AndroidNativeWindow {}

impl AndroidNativeWindow {
    fn new(env: *mut JNIEnv, surface: jobject) -> Self {
        let android_window = unsafe {
            // Get the ANativeWindow associated with the Android Surface object
            // so that it can be used by Rust.
            //
            // This function will automatically increase its reference count by 1
            // when returning ANativeWindow to prevent the object from being
            // accidentally released on the Android side.
            ndk_sys::ANativeWindow_fromSurface(env as *mut _, surface)
        };

        Self { android_window }
    }

    fn width(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getWidth(self.android_window) as u32 }
    }

    fn height(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getHeight(self.android_window) as u32 }
    }
}

impl Drop for AndroidNativeWindow {
    fn drop(&mut self) {
        unsafe {
            ndk_sys::ANativeWindow_release(self.android_window);
        }
    }
}

impl HasWindowHandle for AndroidNativeWindow {
    fn window_handle(&self) -> Result<WindowHandle, HandleError> {
        unsafe {
            let handle = AndroidNdkWindowHandle::new(
                std::ptr::NonNull::new(self.android_window as *mut _).unwrap(),
            );
            Ok(WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(
                handle,
            )))
        }
    }
}

impl HasDisplayHandle for AndroidNativeWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe {
            Ok(DisplayHandle::borrow_raw(RawDisplayHandle::Android(
                AndroidDisplayHandle::new(),
            )))
        }
    }
}

/////

#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct Renderer {
    wrapped: Arc<crate::Renderer>,
}

async fn headless() -> crate::Renderer {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: BACKENDS,
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = crate::platform::all::request_device(&adapter).await;

    crate::Renderer::new(device, queue)
}

#[cfg_attr(mobile, uniffi::export)]
impl Renderer {
    #[cfg_attr(mobile, uniffi::constructor)]
    pub async fn headless() -> Self {
        Renderer {
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
            .map(|it| it.removing_padding())
            .map(|it| it.into())
    }
}

#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct Stage {
    surface: Option<wgpu::Surface<'static>>,
    wrapped: Arc<crate::Stage>,
}

/// NOTE: Stage needs 2 raw pointers to connect with:
///  - the JNI environment
///  - the Android Surface object
///
/// Unfortunately uniffi currently does not support interfacing with raw
/// pointers. In addition, in order to get the JNIEnv pointer, we need to
/// use a proper JNI function, which is not possible to do with uniffi.
///
/// So in the end we do not expose this function and wrap it into a ugly raw ffi
impl Stage {
    pub async fn in_surface(env: *mut JNIEnv<'_>, surface: jobject) -> Self {
        let window = AndroidNativeWindow::new(env, surface);
        let window_width = window.width();
        let window_height = window.height();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKENDS,
            ..Default::default()
        });

        let handle: Box<dyn wgpu::WindowHandle> = Box::new(window);
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Window(handle))
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = crate::platform::all::request_device(&adapter).await;

        let capabilitiess = surface.get_capabilities(&adapter);
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilitiess.formats[0].remove_srgb_suffix(),
            width: u32::max(window_width, 1),
            height: u32::max(window_height, 1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilitiess.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let stage = crate::Stage::new(crate::Renderer::new(device, queue));
        surface.configure(stage.device(), &surface_configuration);

        Self {
            surface: Some(surface),
            wrapped: stage.into(),
        }
    }
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

#[no_mangle]
#[jni_fn("com.photoroom.engine.StageExtensions")]
pub fn pg_stage_create_in_surface(env: *mut JNIEnv, _: JClass, surface: jobject) -> *const Stage {
    let stage = pollster::block_on(Stage::in_surface(env, surface));
    Arc::into_raw(Arc::new(stage))
}
