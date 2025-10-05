use jni::{JNIEnv, objects::JClass, sys::jobject};
use jni_fn::jni_fn;
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};

use crate::{Target, TargetFrame, WindowTarget};

/// An implementation of HasWindowHandle + HasDisplayHandle for Android.
#[derive(Debug)]
struct AndroidNativeWindow {
    android_window: *mut ndk_sys::ANativeWindow,
}

unsafe impl Send for AndroidNativeWindow {}
unsafe impl Sync for AndroidNativeWindow {}

impl AndroidNativeWindow {
    fn new(env: *mut JNIEnv, surface: jobject) -> Self {
        let android_window = unsafe { ndk_sys::ANativeWindow_fromSurface(env as *mut _, surface) };
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
        unsafe { ndk_sys::ANativeWindow_release(self.android_window) }
    }
}

impl HasWindowHandle for AndroidNativeWindow {
    fn window_handle(&self) -> Result<WindowHandle, HandleError> {
        unsafe {
            let nn = match std::ptr::NonNull::new(self.android_window as *mut _) {
                Some(p) => p,
                None => return Err(HandleError::Unavailable),
            };
            let handle = AndroidNdkWindowHandle::new(nn);
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

#[lsp_doc("docs/api/hidden/platforms/android/android_target/android_target.md")]
#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct AndroidTarget(WindowTarget);

#[lsp_doc("docs/api/hidden/platforms/android/android_texture_target/android_texture_target.md")]
#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct AndroidTextureTarget(crate::TextureTarget);

impl Target for AndroidTarget {
    fn size(&self) -> crate::Size {
        self.0.size()
    }
    fn resize(&mut self, size: impl Into<crate::Size>) {
        self.0.resize(size.into());
    }
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.0.get_current_frame()
    }
}

impl Target for AndroidTextureTarget {
    fn size(&self) -> crate::Size {
        self.0.size()
    }
    fn resize(&mut self, size: impl Into<crate::Size>) {
        self.0.resize(size.into());
    }
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, wgpu::SurfaceError> {
        self.0.get_current_frame()
    }
    fn get_image(&self) -> Vec<u8> {
        <crate::TextureTarget as Target>::get_image(&self.0)
    }
}

#[cfg_attr(mobile, uniffi::export)]
impl crate::Renderer {
    /// Creates a new Renderer (Android wrapper variant)
    #[lsp_doc("docs/api/core/renderer/new.md")]
    pub fn new_android() -> Self {
        Self::new()
    }

    /// Create a target from Android Surface (JNI env + surface jobject)
    #[lsp_doc("docs/api/core/renderer/create_target.md")]
    pub async fn create_target_android(
        &self,
        env: *mut JNIEnv,
        surface: jobject,
    ) -> Result<AndroidTarget, crate::InitializationError> {
        let window = AndroidNativeWindow::new(env, surface);
        let size = wgpu::Extent3d {
            width: u32::max(window.width(), 1),
            height: u32::max(window.height(), 1),
            depth_or_array_layers: 1,
        };

        // Create surface via Renderer helper
        let handle: Box<dyn wgpu::WindowHandle> = Box::new(window);
        let (context, surface, config) = self
            .create_surface(wgpu::SurfaceTarget::Window(handle), size)
            .await?;
        Ok(AndroidTarget(WindowTarget::new(context, surface, config)))
    }

    /// Headless texture target (Android wrapper variant)
    #[lsp_doc("docs/api/core/renderer/create_texture_target.md")]
    pub async fn create_texture_target_android(
        &self,
        size: impl Into<crate::Size>,
    ) -> Result<AndroidTextureTarget, crate::InitializationError> {
        let target = self.create_texture_target(size).await?;
        Ok(AndroidTextureTarget(target))
    }

    /// Render wrapper (Android variant)
    #[lsp_doc("docs/api/core/renderer/render.md")]
    pub fn render_android(
        &self,
        renderable: &impl crate::renderer::Renderable,
        target: &impl crate::Target,
    ) -> Result<(), crate::RendererError> {
        self.render(renderable, target)
    }
}

// Optional raw JNI entry point that returns a pointer to AndroidTarget for non-uniffi paths.
#[no_mangle]
#[jni_fn("org.fragmentcolor.Renderer")]
pub fn create_target(env: *mut JNIEnv, _class: JClass, surface: jobject) -> *const AndroidTarget {
    let renderer = crate::Renderer::new();
    match pollster::block_on(renderer.create_target_android(env, surface)) {
        Ok(target) => Box::into_raw(Box::new(target)),
        Err(_) => std::ptr::null(),
    }
}
