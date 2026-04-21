//! Android-specific FFI bridge.
//!
//! Uniffi cannot marshal `JNIEnv*` + `jobject`, so the Kotlin-facing
//! `Surface → WindowTarget` constructor is exposed as a raw `#[jni_fn]`
//! entry point that returns an `Arc<WindowTarget>` pointer. The generated
//! Kotlin bindings then wrap it via `WindowTarget(Pointer(ptr))` (the JNA
//! pointer-constructor that uniffi's Kotlin backend emits for every
//! `uniffi::Object`).

use std::sync::Arc;

use jni::{JNIEnv, objects::JClass, sys::jobject};
use jni_fn::jni_fn;
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, HandleError, HasDisplayHandle,
    HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};

use crate::{Renderer, WindowTarget};

/// HasWindowHandle + HasDisplayHandle wrapper over an `ANativeWindow`.
#[derive(Debug)]
struct AndroidNativeWindow {
    window: *mut ndk_sys::ANativeWindow,
}

unsafe impl Send for AndroidNativeWindow {}
unsafe impl Sync for AndroidNativeWindow {}

impl AndroidNativeWindow {
    fn from_surface(env: *mut JNIEnv, surface: jobject) -> Self {
        // ANativeWindow_fromSurface retains the surface; Drop releases it.
        let window = unsafe { ndk_sys::ANativeWindow_fromSurface(env as *mut _, surface) };
        Self { window }
    }

    fn width(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getWidth(self.window) as u32 }
    }

    fn height(&self) -> u32 {
        unsafe { ndk_sys::ANativeWindow_getHeight(self.window) as u32 }
    }
}

impl Drop for AndroidNativeWindow {
    fn drop(&mut self) {
        unsafe { ndk_sys::ANativeWindow_release(self.window) }
    }
}

impl HasWindowHandle for AndroidNativeWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let nn = std::ptr::NonNull::new(self.window as *mut _).ok_or(HandleError::Unavailable)?;
        let handle = AndroidNdkWindowHandle::new(nn);
        // SAFETY: NativeWindow is alive for the duration of `&self`.
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AndroidNdk(handle)) })
    }
}

impl HasDisplayHandle for AndroidNativeWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // SAFETY: AndroidDisplayHandle carries no pointer; always valid.
        Ok(unsafe {
            DisplayHandle::borrow_raw(RawDisplayHandle::Android(AndroidDisplayHandle::new()))
        })
    }
}

async fn build_window_target(env: *mut JNIEnv<'_>, surface: jobject) -> Option<Arc<WindowTarget>> {
    let window = AndroidNativeWindow::from_surface(env, surface);
    let size = wgpu::Extent3d {
        width: u32::max(window.width(), 1),
        height: u32::max(window.height(), 1),
        depth_or_array_layers: 1,
    };

    let renderer = Renderer::new();
    let handle: Box<dyn wgpu::WindowHandle> = Box::new(window);
    let (context, surface, config) = renderer
        .create_surface(wgpu::SurfaceTarget::Window(handle), size)
        .await
        .ok()?;
    Some(Arc::new(WindowTarget::new(context, surface, config)))
}

/// Raw JNI entry point. Returns `Arc::into_raw(target)` as an opaque pointer
/// (cast to `*const WindowTarget` for jlong compatibility). Caller owns the
/// resulting Arc — the uniffi-generated Kotlin bindings take ownership on
/// `WindowTarget(Pointer(ptr))`.
///
/// Returns `std::ptr::null()` on failure.
#[unsafe(no_mangle)]
#[jni_fn("org.fragmentcolor.RendererJni")]
pub fn create_window_target_from_surface(
    env: *mut JNIEnv,
    _class: JClass,
    surface: jobject,
) -> *const WindowTarget {
    match pollster::block_on(build_window_target(env, surface)) {
        Some(target) => Arc::into_raw(target),
        None => std::ptr::null(),
    }
}
